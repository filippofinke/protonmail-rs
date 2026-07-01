//! Mailbox-password (key-passphrase) derivation.

use crate::error::{Error, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use proton_srp::mailbox_password_hash;
use secrecy::{ExposeSecret, SecretString};

/// Derive the PGP key passphrase ("salted key password") from the mailbox
/// password and a base64 key salt. Returns the trailing 31-byte bcrypt hash.
pub fn derive_key_passphrase(password: &SecretString, salt_b64: &str) -> Result<SecretString> {
    let salt = STANDARD
        .decode(salt_b64)
        .map_err(|e| Error::Crypto(format!("invalid key salt base64: {e}")))?;
    tracing::trace!(target: "proton_core::crypto", salt_bytes = salt.len(), "deriving key passphrase (bcrypt mailbox hash, last 31 bytes)");
    let hashed = mailbox_password_hash(password.expose_secret(), &salt)
        .map_err(|e| Error::Srp(format!("mailbox password hash failed: {e}")))?;
    let pass = std::str::from_utf8(hashed.hashed_password())
        .map_err(|e| Error::Crypto(format!("non-utf8 key passphrase: {e}")))?;
    tracing::trace!(target: "proton_core::crypto", len = pass.len(), "key passphrase derived");
    Ok(SecretString::from(pass.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_31_byte_passphrase_deterministically() {
        // 16-byte salt, base64-encoded.
        let salt_b64 = STANDARD.encode([7u8; 16]);
        let pw = SecretString::from("hunter2".to_string());
        let a = derive_key_passphrase(&pw, &salt_b64).unwrap();
        let b = derive_key_passphrase(&pw, &salt_b64).unwrap();
        assert_eq!(a.expose_secret().len(), 31);
        assert_eq!(
            a.expose_secret(),
            b.expose_secret(),
            "KDF must be deterministic"
        );

        // Different password → different passphrase.
        let c = derive_key_passphrase(&SecretString::from("other".to_string()), &salt_b64).unwrap();
        assert_ne!(a.expose_secret(), c.expose_secret());
    }

    #[test]
    fn rejects_bad_salt() {
        let pw = SecretString::from("x".to_string());
        assert!(derive_key_passphrase(&pw, "!!!not base64!!!").is_err());
    }
}
