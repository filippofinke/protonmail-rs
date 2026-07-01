//! Session model + on-disk persistence (non-secret metadata file + secrets in
//! the `SecretStore`).

pub mod secret_store;

pub use secret_store::{KeyringStore, MemoryStore, SecretStore};

use crate::error::{Error, Result};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const K_ACCESS: &str = "access_token";
const K_REFRESH: &str = "refresh_token";
const K_SKP: &str = "skp";

/// Auth tokens held in memory.
#[derive(Clone)]
pub struct Tokens {
    /// Session UID.
    pub uid: String,
    /// Access token.
    pub access: SecretString,
    /// Refresh token.
    pub refresh: SecretString,
}

/// Non-secret session metadata persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Session {
    /// Session UID.
    pub uid: String,
    /// App-version string sent on every request.
    pub app_version: String,
    /// API base URL for this session.
    pub base_url: String,
    /// Password mode (1 = single-password, 2 = two-password login).
    #[serde(default = "default_password_mode")]
    pub password_mode: u8,
    /// Optional `User-Agent` recorded for this session.
    #[serde(default)]
    pub user_agent: Option<String>,
}

fn default_password_mode() -> u8 {
    1
}

/// A fully-loaded session (metadata + secrets).
pub struct LoadedSession {
    /// Non-secret session metadata.
    pub session: Session,
    /// Auth tokens loaded from the secret store.
    pub tokens: Tokens,
    /// Secret key passphrase loaded from the secret store.
    pub skp: SecretString,
}

/// Resolves on-disk paths for sessions (overridable in tests).
#[derive(Clone)]
pub struct Paths {
    base: PathBuf,
}

impl Paths {
    /// Platform config dir: `<config>/protonmail-cli`.
    pub fn system() -> Result<Self> {
        let dirs = directories::ProjectDirs::from("me", "Proton", "protonmail-cli")
            .ok_or_else(|| Error::Session("cannot resolve config dir".into()))?;
        Ok(Paths {
            base: dirs.config_dir().to_path_buf(),
        })
    }
    /// Resolve paths under an explicit base directory (used in tests).
    pub fn with_base(base: impl Into<PathBuf>) -> Self {
        Paths { base: base.into() }
    }
    fn sessions_dir(&self) -> PathBuf {
        self.base.join("sessions")
    }
    fn session_file(&self, profile: &str) -> PathBuf {
        let p = if profile.is_empty() {
            "default"
        } else {
            profile
        };
        self.sessions_dir().join(format!("{p}.json"))
    }
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    Ok(())
}
#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}

impl Session {
    /// Persist metadata to disk and secrets to the store.
    pub fn save(
        &self,
        paths: &Paths,
        profile: &str,
        store: &dyn SecretStore,
        tokens: &Tokens,
        skp: &SecretString,
    ) -> Result<()> {
        let dir = paths.sessions_dir();
        std::fs::create_dir_all(&dir)?;
        set_mode(&dir, 0o700)?;
        let file = paths.session_file(profile);
        let json = serde_json::to_vec_pretty(self)?;
        std::fs::write(&file, json)?;
        set_mode(&file, 0o600)?;

        store.set(K_ACCESS, tokens.access.expose_secret())?;
        store.set(K_REFRESH, tokens.refresh.expose_secret())?;
        store.set(K_SKP, skp.expose_secret())?;
        Ok(())
    }

    /// Persist only the rotated tokens (after a refresh).
    pub fn save_tokens(store: &dyn SecretStore, access: &str, refresh: &str) -> Result<()> {
        store.set(K_ACCESS, access)?;
        store.set(K_REFRESH, refresh)?;
        Ok(())
    }

    /// Load a session if present and complete.
    pub fn load(
        paths: &Paths,
        profile: &str,
        store: &dyn SecretStore,
    ) -> Result<Option<LoadedSession>> {
        let file = paths.session_file(profile);
        let bytes = match std::fs::read(&file) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let session: Session = match serde_json::from_slice(&bytes) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };
        let (access, refresh, skp) = match (
            store.get(K_ACCESS)?,
            store.get(K_REFRESH)?,
            store.get(K_SKP)?,
        ) {
            (Some(a), Some(r), Some(s)) => (a, r, s),
            _ => return Ok(None),
        };
        let tokens = Tokens {
            uid: session.uid.clone(),
            access: SecretString::from(access),
            refresh: SecretString::from(refresh),
        };
        Ok(Some(LoadedSession {
            session,
            tokens,
            skp: SecretString::from(skp),
        }))
    }

    /// Remove the on-disk file and stored secrets.
    pub fn clear(paths: &Paths, profile: &str, store: &dyn SecretStore) -> Result<()> {
        let file = paths.session_file(profile);
        match std::fs::remove_file(&file) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        store.delete(K_ACCESS)?;
        store.delete(K_REFRESH)?;
        store.delete(K_SKP)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_base() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("proton-test-{}-{}", std::process::id(), nanos))
    }

    #[test]
    fn save_load_clear_roundtrip() {
        let paths = Paths::with_base(unique_base());
        let store = MemoryStore::new();
        let s = Session {
            uid: "UID1".into(),
            app_version: "Other".into(),
            base_url: "https://mail.proton.me/api".into(),
            password_mode: 1,
            user_agent: None,
        };
        let tokens = Tokens {
            uid: "UID1".into(),
            access: SecretString::from("acc"),
            refresh: SecretString::from("ref"),
        };
        s.save(
            &paths,
            "default",
            &store,
            &tokens,
            &SecretString::from("skp-secret"),
        )
        .unwrap();

        let loaded = Session::load(&paths, "default", &store).unwrap().unwrap();
        assert_eq!(loaded.session.uid, "UID1");
        assert_eq!(loaded.session.app_version, "Other");
        assert_eq!(loaded.tokens.access.expose_secret(), "acc");
        assert_eq!(loaded.skp.expose_secret(), "skp-secret");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(paths.session_file("default"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        Session::clear(&paths, "default", &store).unwrap();
        assert!(Session::load(&paths, "default", &store).unwrap().is_none());
    }

    #[test]
    fn load_missing_returns_none() {
        let paths = Paths::with_base(unique_base());
        let store = MemoryStore::new();
        assert!(Session::load(&paths, "nope", &store).unwrap().is_none());
    }
}
