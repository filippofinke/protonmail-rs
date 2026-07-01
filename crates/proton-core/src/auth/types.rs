//! Auth wire types.

use serde::Deserialize;

/// Response from creating an unauthenticated session (UID + initial tokens).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionResp {
    /// Session UID identifying this (initially unauthenticated) session.
    #[serde(rename = "UID")]
    pub uid: String,
    /// Bearer access token for the session.
    pub access_token: String,
    /// Refresh token used to mint new access tokens.
    pub refresh_token: String,
}

/// SRP challenge parameters returned by the auth-info endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthInfo {
    /// Signed SRP group modulus (its signature is verified during login).
    pub modulus: String,
    /// Server's SRP ephemeral value (B).
    pub server_ephemeral: String,
    /// SRP hash version to use.
    pub version: u8,
    /// Per-user SRP salt.
    pub salt: String,
    /// Opaque SRP session identifier echoed back when submitting the proof.
    #[serde(rename = "SRPSession")]
    pub srp_session: String,
}

/// Two-factor status block embedded in the auth response.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TwoFa {
    /// Bitmask of enabled 2FA methods: bit 0 = TOTP, bit 1 = FIDO2/WebAuthn.
    #[serde(default)]
    pub enabled: u32,
}

/// Response from completing SRP authentication (tokens, server proof, 2FA/scope).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthResponse {
    /// Session UID for the now-authenticated session.
    #[serde(rename = "UID")]
    pub uid: String,
    /// Bearer access token for the session.
    pub access_token: String,
    /// Refresh token used to mint new access tokens.
    pub refresh_token: String,
    /// Server's SRP proof, verified client-side as a MITM guard.
    #[serde(default)]
    pub server_proof: String,
    /// Two-factor status (which methods are required).
    #[serde(rename = "2FA", default)]
    pub two_fa: TwoFa,
    /// Password mode: `1` for single-password, `2` for separate mailbox password.
    #[serde(default = "one")]
    pub password_mode: u8,
    /// Granted scope string, if returned.
    #[serde(default)]
    pub scope: Option<String>,
}

fn one() -> u8 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_auth_response() {
        let json = serde_json::json!({
            "Code": 1000, "UID": "x", "AccessToken": "a", "RefreshToken": "r",
            "ServerProof": "sp", "2FA": {"Enabled": 1}, "PasswordMode": 2
        });
        let r: AuthResponse = serde_json::from_value(json).unwrap();
        assert_eq!(r.uid, "x");
        assert_eq!(r.two_fa.enabled, 1);
        assert_eq!(r.password_mode, 2);
    }

    #[test]
    fn auth_info_srp_session_rename() {
        let json = serde_json::json!({
            "Modulus": "m", "ServerEphemeral": "se", "Version": 4, "Salt": "s", "SRPSession": "sess"
        });
        let i: AuthInfo = serde_json::from_value(json).unwrap();
        assert_eq!(i.srp_session, "sess");
        assert_eq!(i.version, 4);
    }
}
