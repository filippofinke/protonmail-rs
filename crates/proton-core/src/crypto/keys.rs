//! Key hierarchy: wire types + unlock into an in-memory `KeyStore`.
//!
//! Because `proton-crypto`'s key/provider types are unnameable (private
//! associated types), the `KeyStore` holds *armored key material + the
//! passphrase that unlocks it*, and each crypto op re-imports as needed.

use super::kdf;
use crate::error::{Error, Result};
use proton_crypto::crypto::{
    DataEncoding, Decryptor, DecryptorSync, PGPProviderSync, VerifiedData, Verifier, VerifierSync,
};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

/// A raw API key (user or address).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiKey {
    /// The key's API ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Armored PGP private key.
    pub private_key: String,
    /// Encrypted passphrase (a PGP message to a user key) that unlocks this
    /// address key; `None` for legacy keys unlocked with the mailbox password.
    #[serde(default)]
    pub token: Option<String>,
    /// Detached signature over the token, verified against the user keys.
    #[serde(default)]
    pub signature: Option<String>,
    /// Activation token for a not-yet-activated key, if present.
    #[serde(default)]
    pub activation: Option<String>,
    /// `1` if this is the primary key in its ring.
    #[serde(default)]
    pub primary: u8,
    /// `1` if the key is active (usable); inactive (`0`) keys are skipped.
    #[serde(default = "default_active")]
    pub active: u8,
    /// Key capability flags (encrypt/sign), if present.
    #[serde(default)]
    pub flags: Option<u32>,
}

fn default_active() -> u8 {
    1
}

/// The account's user-level key ring.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    /// User keys (the root of the key hierarchy).
    pub keys: Vec<ApiKey>,
}

/// An email address and its key ring.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Address {
    /// The address's API ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// The email address.
    pub email: String,
    /// 1-based sort order from the API; the lowest-order address is the
    /// account's default/primary sending address.
    #[serde(default)]
    pub order: u32,
    /// The address's keys.
    #[serde(default)]
    pub keys: Vec<ApiKey>,
}

/// A per-key salt used to derive that key's passphrase from the mailbox password.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeySalt {
    /// The ID of the key this salt belongs to.
    #[serde(rename = "ID")]
    pub id: String,
    /// Base64-encoded salt; `None` for keys without one.
    pub key_salt: Option<String>,
}

/// An unlocked key, stored as armored material + the passphrase to re-import it.
#[derive(Clone)]
pub struct StoredKey {
    /// Armored PGP private key.
    pub armored: String,
    /// Passphrase that unlocks the armored key on re-import.
    pub passphrase: SecretString,
}

/// Unlocked keys for one address.
#[derive(Clone)]
pub struct AddressKeys {
    /// The address's API ID.
    pub address_id: String,
    /// The email address.
    pub email: String,
    /// The address's unlocked keys.
    pub keys: Vec<StoredKey>,
}

/// All unlocked keys for the account.
#[derive(Clone)]
pub struct KeyStore {
    /// Unlocked user keys.
    pub user_keys: Vec<StoredKey>,
    /// Unlocked keys per address, ascending by API `Order` (default-first).
    pub addresses: Vec<AddressKeys>,
}

impl KeyStore {
    /// Look up an address's keys by address ID.
    pub fn address(&self, address_id: &str) -> Option<&AddressKeys> {
        self.addresses.iter().find(|a| a.address_id == address_id)
    }

    /// Look up an address's keys by email (case-insensitive).
    pub fn address_for_email(&self, email: &str) -> Option<&AddressKeys> {
        let e = email.to_ascii_lowercase();
        self.addresses
            .iter()
            .find(|a| a.email.to_ascii_lowercase() == e)
    }

    /// Default sending address: the first address by API `Order` (matching the
    /// web client's default). `addresses` is built in ascending-order during
    /// unlock, so this is simply the first entry. Pick a different alias by
    /// passing a `from`/`--from` address (resolved via [`Self::address_for_email`]).
    pub fn primary_address(&self) -> Option<&AddressKeys> {
        self.addresses.first()
    }
}

/// Order addresses by ascending API `Order` (the default/primary sending
/// address first). The sort is stable, so addresses with an absent or equal
/// `Order` keep their original API order.
fn addresses_by_order(addresses: &[Address]) -> Vec<&Address> {
    let mut ordered: Vec<&Address> = addresses.iter().collect();
    ordered.sort_by_key(|a| a.order);
    ordered
}

/// Unlock the key hierarchy using the mailbox password + salts (login path).
/// Returns the `KeyStore` and the primary user key's passphrase (`skp`) for
/// session persistence.
pub fn unlock<P: PGPProviderSync>(
    provider: &P,
    user: &User,
    addresses: &[Address],
    password: &SecretString,
    salts: &[KeySalt],
) -> Result<(KeyStore, SecretString)> {
    unlock_inner(provider, user, addresses, |k| {
        let salt_b64 = salts
            .iter()
            .find(|s| s.id == k.id)
            .and_then(|s| s.key_salt.clone())
            .or_else(|| salts.first().and_then(|s| s.key_salt.clone()))
            .ok_or_else(|| Error::Crypto("no key salt available".into()))?;
        kdf::derive_key_passphrase(password, &salt_b64)
    })
}

/// Unlock the key hierarchy using a stored `skp` directly (resume path; no
/// password / KDF). The same passphrase is used for every user key.
pub fn unlock_with_skp<P: PGPProviderSync>(
    provider: &P,
    user: &User,
    addresses: &[Address],
    skp: &SecretString,
) -> Result<KeyStore> {
    Ok(unlock_inner(provider, user, addresses, |_k| Ok(skp.clone()))?.0)
}

fn unlock_inner<P: PGPProviderSync>(
    provider: &P,
    user: &User,
    addresses: &[Address],
    user_pass_for: impl Fn(&ApiKey) -> Result<SecretString>,
) -> Result<(KeyStore, SecretString)> {
    tracing::debug!(target: "proton_core::crypto", user_keys = user.keys.len(), addresses = addresses.len(), "unlocking key hierarchy");
    // --- user keys ---
    let mut user_stored: Vec<StoredKey> = Vec::new();
    let mut user_privs: Vec<P::PrivateKey> = Vec::new();
    for k in &user.keys {
        if k.active == 0 {
            continue;
        }
        let pass = user_pass_for(k)?;
        if let Ok(pk) = provider.private_key_import(
            k.private_key.as_bytes(),
            pass.expose_secret().as_bytes(),
            DataEncoding::Armor,
        ) {
            tracing::trace!(target: "proton_core::crypto", key_id = %k.id, "unlocked user key");
            user_privs.push(pk);
            user_stored.push(StoredKey {
                armored: k.private_key.clone(),
                passphrase: pass,
            });
        }
    }
    if user_privs.is_empty() {
        return Err(Error::Crypto(
            "no user key could be unlocked (wrong password?)".into(),
        ));
    }
    tracing::debug!(target: "proton_core::crypto", count = user_privs.len(), "user keys unlocked");

    // public counterparts for token-signature verification
    let mut user_pubs: Vec<P::PublicKey> = Vec::new();
    for pk in &user_privs {
        if let Ok(pubk) = provider.private_key_to_public_key(pk) {
            user_pubs.push(pubk);
        }
    }
    let fallback_pass = user_stored[0].passphrase.clone();

    // --- address keys ---
    // Build in ascending `Order` so the default/primary sending address is
    // first; `primary_address()` then just takes `.first()`.
    let mut addr_out: Vec<AddressKeys> = Vec::new();
    for a in addresses_by_order(addresses) {
        let mut keys: Vec<StoredKey> = Vec::new();
        for k in &a.keys {
            if k.active == 0 {
                continue;
            }
            let pass: SecretString = match (&k.token, &k.signature) {
                (Some(tok), Some(sig)) => {
                    tracing::trace!(target: "proton_core::crypto", address = %a.email, key_id = %k.id, "address key: decrypting token + verifying signature against user keys");
                    match decrypt_token(provider, &user_privs, &user_pubs, tok, sig) {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::warn!(target: "proton_core::crypto", address = %a.email, key_id = %k.id, error = %e, "address key token invalid — skipping key");
                            continue; // bad/unverifiable token → skip key
                        }
                    }
                }
                _ => {
                    tracing::trace!(target: "proton_core::crypto", address = %a.email, key_id = %k.id, "address key: legacy skp passphrase");
                    fallback_pass.clone()
                }
            };
            if provider
                .private_key_import(
                    k.private_key.as_bytes(),
                    pass.expose_secret().as_bytes(),
                    DataEncoding::Armor,
                )
                .is_ok()
            {
                keys.push(StoredKey {
                    armored: k.private_key.clone(),
                    passphrase: pass,
                });
            }
        }
        if !keys.is_empty() {
            tracing::debug!(target: "proton_core::crypto", address = %a.email, keys = keys.len(), "address keys unlocked");
            addr_out.push(AddressKeys {
                address_id: a.id.clone(),
                email: a.email.clone(),
                keys,
            });
        }
    }
    if addr_out.is_empty() {
        return Err(Error::Crypto("no address key could be unlocked".into()));
    }
    tracing::info!(target: "proton_core::crypto", addresses = addr_out.len(), "key hierarchy unlocked");

    let primary_skp = user_stored[0].passphrase.clone();
    Ok((
        KeyStore {
            user_keys: user_stored,
            addresses: addr_out,
        },
        primary_skp,
    ))
}

/// Decrypt an address-key token (PGP message to the user key) and verify its
/// detached signature with the user keys. Returns the token bytes (the address
/// key passphrase). Verification failure is an error (the key is then skipped).
fn decrypt_token<P: PGPProviderSync>(
    provider: &P,
    user_privs: &[P::PrivateKey],
    user_pubs: &[P::PublicKey],
    token_armored: &str,
    sig_armored: &str,
) -> Result<SecretString> {
    let verified = provider
        .new_decryptor()
        .with_decryption_keys(user_privs.iter())
        .decrypt(token_armored.as_bytes(), DataEncoding::Armor)
        .map_err(|e| Error::Crypto(format!("token decrypt failed: {e}")))?;
    let bytes = verified.as_bytes().to_vec();

    let result = provider
        .new_verifier()
        .with_verification_keys(user_pubs.iter())
        .verify_detached(&bytes, sig_armored.as_bytes(), DataEncoding::Armor);
    if result.is_err() {
        return Err(Error::Crypto("address key token signature invalid".into()));
    }

    let s =
        std::str::from_utf8(&bytes).map_err(|e| Error::Crypto(format!("non-utf8 token: {e}")))?;
    Ok(SecretString::from(s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_address_is_first() {
        // `addresses` is built ascending-by-`Order` during unlock, so the
        // primary/default sending address is simply the first entry.
        let ks = KeyStore {
            user_keys: vec![],
            addresses: vec![
                AddressKeys {
                    address_id: "1".into(),
                    email: "me@protonmail.ch".into(),
                    keys: vec![],
                },
                AddressKeys {
                    address_id: "2".into(),
                    email: "me@pm.me".into(),
                    keys: vec![],
                },
            ],
        };
        assert_eq!(ks.primary_address().unwrap().address_id, "1");
        assert_eq!(ks.primary_address().unwrap().email, "me@protonmail.ch");
    }

    #[test]
    fn addresses_by_order_sorts_ascending_stably() {
        // Exercises the shipped ordering used by `unlock_inner`: an out-of-order
        // API response yields default-first addresses, and equal `Order` keeps
        // the original API sequence.
        let addrs = [
            Address {
                id: "b".into(),
                email: "second@pm.me".into(),
                order: 2,
                keys: vec![],
            },
            Address {
                id: "a".into(),
                email: "first@protonmail.ch".into(),
                order: 1,
                keys: vec![],
            },
            Address {
                id: "c".into(),
                email: "tie@proton.me".into(),
                order: 1,
                keys: vec![],
            },
        ];
        // Input order is [b(2), a(1), c(1)]; a stable ascending sort yields
        // [a(1), c(1), b(2)] — the two `order == 1` entries keep their API order.
        let ordered = addresses_by_order(&addrs);
        assert_eq!(ordered[0].id, "a");
        assert_eq!(ordered[0].email, "first@protonmail.ch");
        assert_eq!(ordered[1].id, "c");
        assert_eq!(ordered[2].id, "b");
    }

    #[test]
    fn apikey_active_defaults_to_one() {
        let k: ApiKey = serde_json::from_value(serde_json::json!({
            "ID": "k1", "PrivateKey": "ARM"
        }))
        .unwrap();
        assert_eq!(k.active, 1);
    }
}
