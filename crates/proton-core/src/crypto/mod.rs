//! Crypto facade over `proton-crypto` (rustpgp backend) + `proton-srp`.

pub mod kdf;
pub mod keys;
pub mod message;

pub use keys::{Address, AddressKeys, ApiKey, KeySalt, KeyStore, StoredKey, User};
pub use message::{
    algo_name, decrypt_attachment, decrypt_body, encrypt_attachment, encrypt_for_transport,
    encrypt_self_draft, encrypt_text_with_password, new_session_key, rewrap_attachment_session_key,
    wrap_session_key, wrap_session_key_to_self, wrap_session_key_with_password, SessionKeyMaterial,
    UploadedAttachment, Verdict,
};

/// Obtain a pure-Rust PGP provider (rustpgp backend; no cgo).
pub fn provider() -> impl proton_crypto::crypto::PGPProviderSync {
    proton_crypto::new_pgp_provider()
}
