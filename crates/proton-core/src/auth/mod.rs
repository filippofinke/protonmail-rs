//! SRP login, 2FA, refresh, and logout.

pub mod types;

use crate::error::{Error, Result};
use crate::session::Tokens;
use crate::transport::{Doer, HttpClient, Request};
use proton_srp::{RPGPVerifier, SRPAuth, SRPProofB64, SrpHashVersion};
use secrecy::{ExposeSecret, SecretString};
use types::{AuthInfo, AuthResponse, SessionResp};

/// Result of a successful login.
pub struct LoginResult {
    /// Session tokens (UID, access, refresh) for the authenticated session.
    pub tokens: Tokens,
    /// Password mode: `1` for single-password, `2` for separate mailbox password.
    pub password_mode: u8,
}

/// Perform the full SRP login (+ 2FA if required).
pub async fn login(
    http: &HttpClient,
    username: &str,
    password: &SecretString,
    totp: Option<&str>,
) -> Result<LoginResult> {
    tracing::info!(target: "proton_core::auth", username, "login: starting SRP flow");

    // 1. Unauthenticated session.
    tracing::debug!(target: "proton_core::auth", "login step 1/4: creating unauthenticated session (POST /auth/v4/sessions)");
    let sess: SessionResp = http
        .decode(
            Request::post("/auth/v4/sessions")
                .json(serde_json::json!({}))
                .enforce_unauth()
                .no_refresh(),
        )
        .await?;
    tracing::debug!(target: "proton_core::auth", uid = %sess.uid, "login: got unauth session");
    http.set_tokens(
        sess.uid,
        SecretString::from(sess.access_token),
        SecretString::from(sess.refresh_token),
    )
    .await;

    // 2. SRP challenge.
    tracing::debug!(target: "proton_core::auth", "login step 2/4: fetching SRP challenge (POST /core/v4/auth/info)");
    let info: AuthInfo = http
        .decode(
            Request::post("/core/v4/auth/info")
                .json(serde_json::json!({ "Username": username }))
                .no_refresh(),
        )
        .await?;
    tracing::debug!(target: "proton_core::auth", srp_version = info.version, salt_len = info.salt.len(), modulus_len = info.modulus.len(), "login: received modulus, salt, server ephemeral");

    // 3. Compute SRP proofs (modulus signature verified internally by RPGPVerifier).
    let version = SrpHashVersion::try_from(info.version)
        .map_err(|e| Error::Srp(format!("unsupported SRP version {}: {e}", info.version)))?;
    tracing::debug!(target: "proton_core::auth", "login step 3/4: verifying signed modulus + generating client proof (proton-srp)");
    let verifier = RPGPVerifier::default();
    let srp = SRPAuth::new(
        &verifier,
        Some(username),
        password.expose_secret(),
        version,
        &info.salt,
        &info.modulus,
        &info.server_ephemeral,
    )
    .map_err(|e| Error::Srp(format!("SRP setup failed: {e}")))?;
    let proof = srp
        .generate_proofs()
        .map_err(|e| Error::Srp(format!("SRP proof failed: {e}")))?;
    tracing::debug!(target: "proton_core::auth", "login: client proof + ephemeral generated; modulus signature verified");
    let b64: SRPProofB64 = proof.into();

    // 4. Submit proof.
    tracing::debug!(target: "proton_core::auth", "login step 4/4: submitting client proof (POST /core/v4/auth)");
    let resp: AuthResponse = http
        .decode(
            Request::post("/core/v4/auth")
                .json(serde_json::json!({
                    "Username": username,
                    "ClientProof": b64.client_proof,
                    "ClientEphemeral": b64.client_ephemeral,
                    "SRPSession": info.srp_session,
                }))
                .no_refresh(),
        )
        .await?;

    // 5. Verify the server proof (MITM guard).
    if !b64.compare_server_proof(&resp.server_proof) {
        tracing::error!(target: "proton_core::auth", "login: SERVER PROOF MISMATCH — aborting (possible MITM)");
        return Err(Error::Srp("server proof verification failed".into()));
    }
    tracing::info!(target: "proton_core::auth", uid = %resp.uid, password_mode = resp.password_mode, two_fa = resp.two_fa.enabled, "login: server proof verified; authenticated");

    // Promote to the authenticated session.
    http.set_tokens(
        resp.uid.clone(),
        SecretString::from(resp.access_token.clone()),
        SecretString::from(resp.refresh_token.clone()),
    )
    .await;

    // 6. 2FA if required. TOTP is bit 0; FIDO2/WebAuthn is bit 1.
    if resp.two_fa.enabled & 1 == 0 && resp.two_fa.enabled & 2 != 0 {
        return Err(Error::Other(
            "this account requires a security key (FIDO2/WebAuthn) for 2FA, which is not yet \
             supported — enable a TOTP authenticator app, or use an app/bridge password"
                .into(),
        ));
    }
    if resp.two_fa.enabled & 1 != 0 {
        tracing::debug!(target: "proton_core::auth", "login: 2FA required — submitting TOTP (POST /core/v4/auth/2fa)");
        let code = totp.ok_or_else(|| {
            Error::Other("account requires 2FA but no TOTP code was provided".into())
        })?;
        let _: serde_json::Value = http
            .decode(
                Request::post("/core/v4/auth/2fa")
                    .json(serde_json::json!({ "TwoFactorCode": code }))
                    .no_refresh(),
            )
            .await?;
        tracing::debug!(target: "proton_core::auth", "login: 2FA accepted");
    }
    tracing::info!(target: "proton_core::auth", "login: complete");

    Ok(LoginResult {
        tokens: Tokens {
            uid: resp.uid,
            access: SecretString::from(resp.access_token),
            refresh: SecretString::from(resp.refresh_token),
        },
        password_mode: resp.password_mode,
    })
}

/// Revoke the current session server-side.
pub async fn logout(http: &HttpClient) -> Result<()> {
    let _: serde_json::Value = http.decode(Request::delete("/core/v4/auth")).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn logout_calls_revoke() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/core/v4/auth"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"Code": 1000})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let http = HttpClient::new(server.uri(), "Other");
        logout(&http).await.unwrap();
    }
}
