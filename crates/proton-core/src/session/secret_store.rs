//! Secret storage abstraction: an OS-keychain impl for production and an
//! in-memory impl for tests.

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::Mutex;

/// Storage for per-profile secrets (access token, refresh token, key passphrase).
pub trait SecretStore: Send + Sync {
    /// Store `value` under `key`, overwriting any existing entry.
    fn set(&self, key: &str, value: &str) -> Result<()>;
    /// Retrieve the value stored under `key`, or `None` if absent.
    fn get(&self, key: &str) -> Result<Option<String>>;
    /// Delete the entry stored under `key` (no-op if absent).
    fn delete(&self, key: &str) -> Result<()>;
}

/// OS keychain store (`keyring`): macOS Keychain / Secret Service / WinCred.
pub struct KeyringStore {
    service: String,
    profile: String,
}

impl KeyringStore {
    /// Create a keychain-backed store for the given profile.
    pub fn new(profile: impl Into<String>) -> Self {
        KeyringStore {
            service: "protonmail-cli".into(),
            profile: profile.into(),
        }
    }
    fn account(&self, key: &str) -> String {
        format!("{}:{}", self.profile, key)
    }
}

impl SecretStore for KeyringStore {
    fn set(&self, key: &str, value: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, &self.account(key))
            .map_err(|e| Error::Session(format!("keyring open: {e}")))?;
        entry
            .set_password(value)
            .map_err(|e| Error::Session(format!("keyring set: {e}")))
    }
    fn get(&self, key: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(&self.service, &self.account(key))
            .map_err(|e| Error::Session(format!("keyring open: {e}")))?;
        match entry.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::Session(format!("keyring get: {e}"))),
        }
    }
    fn delete(&self, key: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, &self.account(key))
            .map_err(|e| Error::Session(format!("keyring open: {e}")))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::Session(format!("keyring delete: {e}"))),
        }
    }
}

/// In-memory store for tests.
#[derive(Default)]
pub struct MemoryStore {
    map: Mutex<HashMap<String, String>>,
}

impl MemoryStore {
    /// Create an empty in-memory store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MemoryStore {
    fn set(&self, key: &str, value: &str) -> Result<()> {
        self.map
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }
    fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.map.lock().unwrap().get(key).cloned())
    }
    fn delete(&self, key: &str) -> Result<()> {
        self.map.lock().unwrap().remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_roundtrip() {
        let s = MemoryStore::new();
        assert_eq!(s.get("k").unwrap(), None);
        s.set("k", "v").unwrap();
        assert_eq!(s.get("k").unwrap(), Some("v".to_string()));
        s.delete("k").unwrap();
        assert_eq!(s.get("k").unwrap(), None);
    }
}
