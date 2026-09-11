//! SRP login, 2FA, refresh, and logout.

pub mod types;

use crate::error::{Error, Result};
use crate::session::Tokens;
use crate::transport::{Doer, HttpClient, Request};
use futures::future::BoxFuture;
use proton_srp::{RPGPVerifier, SRPAuth, SRPProofB64, SrpHashVersion};
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use types::{AuthInfo, AuthResponse, SessionResp};

/// Asks the user for a TOTP code during login.
///
/// Called only when the account requires TOTP and no code was supplied up
/// front, after any human verification, so the code is still fresh. Obtain the
/// code interactively and do not persist it.
pub type TotpPrompt = Arc<dyn Fn() -> BoxFuture<'static, Result<String>> + Send + Sync>;

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
    login_with_prompt(http, username, password, totp, None).await
}

/// Like [`login`], but asks `totp_prompt` for the code when the account
/// requires TOTP and `totp` is `None`.
pub async fn login_with_prompt(
    http: &HttpClient,
    username: &str,
    password: &SecretString,
    totp: Option<&str>,
    totp_prompt: Option<&TotpPrompt>,
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

    // 6. 2FA if required.
    second_factor(http, resp.two_fa.enabled, totp, totp_prompt).await?;
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

/// Submit the second factor if the account requires one. `enabled` is the 2FA
/// bitmask: TOTP is bit 0; FIDO2/WebAuthn is bit 1.
async fn second_factor(
    http: &HttpClient,
    enabled: u32,
    totp: Option<&str>,
    totp_prompt: Option<&TotpPrompt>,
) -> Result<()> {
    if enabled & 1 == 0 && enabled & 2 != 0 {
        return Err(Error::Other(
            "this account requires a security key (FIDO2/WebAuthn) for 2FA, which is not yet \
             supported — enable a TOTP authenticator app, or use an app/bridge password"
                .into(),
        ));
    }
    if enabled & 1 == 0 {
        return Ok(());
    }
    let code = match (totp, totp_prompt) {
        (Some(code), _) => code.to_string(),
        (None, Some(prompt)) => prompt().await?,
        (None, None) => {
            return Err(Error::Other(
                "account requires 2FA but no TOTP code was provided".into(),
            ))
        }
    };
    tracing::debug!(target: "proton_core::auth", "login: 2FA required — submitting TOTP (POST /core/v4/auth/2fa)");
    let _: serde_json::Value = http
        .decode(
            Request::post("/core/v4/auth/2fa")
                .json(serde_json::json!({ "TwoFactorCode": code }))
                .no_refresh(),
        )
        .await?;
    tracing::debug!(target: "proton_core::auth", "login: 2FA accepted");
    Ok(())
}

/// Revoke the current session server-side.
pub async fn logout(http: &HttpClient) -> Result<()> {
    let _: serde_json::Value = http.decode(Request::delete("/core/v4/auth")).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wiremock::matchers::{body_json, method, path};
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

    /// A prompt returning `code` and counting its invocations.
    fn counting_prompt(code: &'static str) -> (TotpPrompt, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = calls.clone();
        let prompt: TotpPrompt = Arc::new(move || {
            seen.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { Ok(code.to_string()) })
        });
        (prompt, calls)
    }

    async fn expect_2fa(server: &MockServer, code: &str, times: u64) {
        Mock::given(method("POST"))
            .and(path("/core/v4/auth/2fa"))
            .and(body_json(serde_json::json!({ "TwoFactorCode": code })))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"Code": 1000})),
            )
            .expect(times)
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn second_factor_prefers_supplied_code() {
        let server = MockServer::start().await;
        expect_2fa(&server, "123456", 1).await;
        let http = HttpClient::new(server.uri(), "Other");
        let (prompt, calls) = counting_prompt("000000");
        second_factor(&http, 1, Some("123456"), Some(&prompt))
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn second_factor_prompts_when_code_missing() {
        let server = MockServer::start().await;
        expect_2fa(&server, "654321", 1).await;
        let http = HttpClient::new(server.uri(), "Other");
        let (prompt, calls) = counting_prompt("654321");
        second_factor(&http, 1, None, Some(&prompt)).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn second_factor_errors_without_code_or_prompt() {
        let server = MockServer::start().await;
        expect_2fa(&server, "", 0).await;
        let http = HttpClient::new(server.uri(), "Other");
        let err = second_factor(&http, 1, None, None).await.unwrap_err();
        assert!(err.to_string().contains("no TOTP code"));
    }

    #[tokio::test]
    async fn second_factor_rejects_fido2_only_without_prompting() {
        let server = MockServer::start().await;
        let http = HttpClient::new(server.uri(), "Other");
        let (prompt, calls) = counting_prompt("222222");
        let err = second_factor(&http, 2, None, Some(&prompt))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("FIDO2"));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn second_factor_skipped_when_not_enabled() {
        let server = MockServer::start().await;
        let http = HttpClient::new(server.uri(), "Other");
        let (prompt, calls) = counting_prompt("111111");
        second_factor(&http, 0, None, Some(&prompt)).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(server.received_requests().await.unwrap().is_empty());
    }
}
