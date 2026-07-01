//! Keys, salts, user, addresses, and recipient public-key lookup.

use crate::crypto::keys::{Address, KeySalt, User};
use crate::error::Result;
use crate::transport::{Doer, Request};
use serde::Deserialize;

#[derive(Deserialize)]
struct SaltsResp {
    #[serde(rename = "KeySalts", default)]
    key_salts: Vec<KeySalt>,
}

#[derive(Deserialize)]
struct UserResp {
    #[serde(rename = "User")]
    user: User,
}

#[derive(Deserialize)]
struct AddressesResp {
    #[serde(rename = "Addresses", default)]
    addresses: Vec<Address>,
}

/// A recipient public key from `/keys/all`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiPublicKey {
    /// Armored public key.
    pub public_key: String,
    /// Key capability flags (encrypt/verify; see `key_flag`).
    #[serde(default)]
    pub flags: u32,
    /// `1` if this is the primary key for the address.
    #[serde(default)]
    pub primary: u8,
    /// Key source (internal, WKD, etc.).
    #[serde(default)]
    pub source: u8,
}

#[derive(Deserialize)]
struct AddrKeys {
    #[serde(rename = "Keys", default)]
    keys: Vec<ApiPublicKey>,
}

#[derive(Deserialize)]
struct AllKeysResp {
    #[serde(rename = "Address", default)]
    address: Option<AddrKeys>,
    #[serde(rename = "Unverified", default)]
    unverified: Option<AddrKeys>,
    #[serde(rename = "RecipientType", default)]
    recipient_type: u8,
}

/// Resolved recipient keys. `keys` are Proton-internal (verified) keys;
/// `unverified` are external/WKD keys (for PGP-to-external sending).
#[derive(Debug, Clone)]
pub struct RecipientKeys {
    /// Recipient type (1 = Proton-internal, 2 = external).
    pub recipient_type: u8,
    /// Proton-internal (verified) keys.
    pub keys: Vec<ApiPublicKey>,
    /// External/WKD (unverified) keys.
    pub unverified: Vec<ApiPublicKey>,
}

impl RecipientKeys {
    /// True if this is a Proton-internal recipient (has verified keys).
    pub fn is_internal(&self) -> bool {
        !self.keys.is_empty()
    }
}

/// Fetch the per-key salts used to derive key passphrases.
pub async fn get_key_salts<D: Doer>(d: &D) -> Result<Vec<KeySalt>> {
    let r: SaltsResp = d.decode(Request::get("/core/v4/keys/salts")).await?;
    Ok(r.key_salts)
}

/// Fetch the authenticated user (including their encrypted user keys).
pub async fn get_user<D: Doer>(d: &D) -> Result<User> {
    let r: UserResp = d.decode(Request::get("/core/v4/users")).await?;
    Ok(r.user)
}

/// Fetch the account's addresses (with their encrypted address keys).
pub async fn get_addresses<D: Doer>(d: &D) -> Result<Vec<Address>> {
    let r: AddressesResp = d.decode(Request::get("/core/v4/addresses")).await?;
    Ok(r.addresses)
}

#[derive(Deserialize)]
struct ModulusResp {
    #[serde(rename = "Modulus")]
    modulus: String,
    #[serde(rename = "ModulusID")]
    modulus_id: String,
}

/// Fetch a fresh SRP modulus (for encrypted-outside verifier generation).
pub async fn get_modulus<D: Doer>(d: &D) -> Result<(String, String)> {
    let r: ModulusResp = d.decode(Request::get("/core/v4/auth/modulus")).await?;
    Ok((r.modulus, r.modulus_id))
}

/// Update an address (display name and/or signature).
pub async fn update_address<D: Doer>(
    d: &D,
    id: &str,
    display_name: Option<&str>,
    signature: Option<&str>,
) -> Result<()> {
    let mut body = serde_json::Map::new();
    if let Some(n) = display_name {
        body.insert("DisplayName".into(), serde_json::json!(n));
    }
    if let Some(s) = signature {
        body.insert("Signature".into(), serde_json::json!(s));
    }
    let _: serde_json::Value = d
        .decode(
            Request::put(format!("/core/v4/addresses/{id}")).json(serde_json::Value::Object(body)),
        )
        .await?;
    Ok(())
}

/// Look up a recipient's public keys (verified and unverified) by email.
pub async fn get_all_public_keys<D: Doer>(d: &D, email: &str) -> Result<RecipientKeys> {
    let r: AllKeysResp = d
        .decode(Request::get("/core/v4/keys/all").query("Email", email))
        .await?;
    Ok(RecipientKeys {
        recipient_type: r.recipient_type,
        keys: r.address.map(|a| a.keys).unwrap_or_default(),
        unverified: r.unverified.map(|a| a.keys).unwrap_or_default(),
    })
}

/// Pick the primary encryption-capable key (Flags & NOT_OBSOLETE), else the
/// first encryption-capable one.
pub fn pick_encryption_key(keys: &[ApiPublicKey]) -> Option<&ApiPublicKey> {
    use crate::model::enums::key_flag;
    keys.iter()
        .find(|k| k.primary == 1 && key_flag::can_encrypt(k.flags))
        .or_else(|| keys.iter().find(|k| key_flag::can_encrypt(k.flags)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(public_key: &str, flags: u32, primary: u8) -> ApiPublicKey {
        ApiPublicKey {
            public_key: public_key.into(),
            flags,
            primary,
            source: 0,
        }
    }

    #[test]
    fn picks_primary_encryption_capable_key() {
        use crate::model::enums::key_flag;
        let cap = key_flag::NOT_OBSOLETE | key_flag::NOT_COMPROMISED; // 3
        let obsolete = key_flag::NOT_COMPROMISED; // 1, can't encrypt
        let keys = vec![
            k("obsolete", obsolete, 1),
            k("secondary", cap, 0),
            k("primary", cap, 1),
        ];
        assert_eq!(pick_encryption_key(&keys).unwrap().public_key, "primary");
        let keys2 = vec![k("obsolete", obsolete, 1), k("cap", cap, 0)];
        assert_eq!(pick_encryption_key(&keys2).unwrap().public_key, "cap");
        assert!(pick_encryption_key(&[k("x", obsolete, 1)]).is_none());
    }

    #[test]
    fn recipient_keys_is_internal() {
        let rk = RecipientKeys {
            recipient_type: 1,
            keys: vec![k("a", 3, 1)],
            unverified: vec![],
        };
        assert!(rk.is_internal());
        let ext = RecipientKeys {
            recipient_type: 2,
            keys: vec![],
            unverified: vec![k("b", 3, 1)],
        };
        assert!(!ext.is_internal());
    }
}
