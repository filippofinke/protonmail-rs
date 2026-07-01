//! High-level mail client assembling transport + crypto + API.

pub mod attachments;
pub mod contacts;
pub mod drafts;
pub mod export;
pub mod filters;
pub mod organize;
pub mod read;
pub mod send;
pub mod sync;

use crate::api;
use crate::auth;
use crate::crypto::{self, keys::KeyStore};
use crate::error::{Error, Result};
use crate::session::{KeyringStore, Paths, SecretStore, Session, Tokens};
use crate::transport::HttpClient;
use secrecy::SecretString;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const DEFAULT_BASE_URL: &str = "https://mail.proton.me/api";
const DEFAULT_APP_VERSION: &str = "Other";

/// Options for an interactive login.
pub struct LoginOptions {
    /// Account username (email address).
    pub username: String,
    /// Account password.
    pub password: String,
    /// Optional TOTP code for two-factor authentication.
    pub totp: Option<String>,
    /// Separate mailbox password for two-password (PasswordMode 2) accounts.
    pub mailbox_password: Option<String>,
    /// Local profile name used to store the session.
    pub profile: String,
    /// Optional API base URL override.
    pub base_url: Option<String>,
    /// Optional app-version string to present to the API.
    pub app_version: Option<String>,
    /// Optional `User-Agent` to present as a real client.
    pub user_agent: Option<String>,
    /// Optional human-verification (CAPTCHA) resolver, invoked on API code 9001.
    pub hv: Option<crate::transport::HvResolver>,
}

/// The Proton Mail client.
pub struct Client {
    http: HttpClient,
    keys: KeyStore,
    paths: Paths,
    profile: String,
    store: Arc<dyn SecretStore>,
    sender_cache: Mutex<HashMap<String, Vec<String>>>,
}

impl Client {
    pub(crate) fn http(&self) -> &HttpClient {
        &self.http
    }
    pub(crate) fn keys(&self) -> &KeyStore {
        &self.keys
    }

    fn wire_refresh(http: &mut HttpClient, store: Arc<dyn SecretStore>) {
        http.set_refresh_persist(Arc::new(move |_uid, access, refresh| {
            let _ = Session::save_tokens(store.as_ref(), access, refresh);
        }));
    }

    /// Interactive login: SRP + 2FA, unlock keys, persist session.
    pub async fn login(opts: LoginOptions) -> Result<Client> {
        let base_url = opts
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let app_version = opts
            .app_version
            .clone()
            .unwrap_or_else(|| DEFAULT_APP_VERSION.to_string());
        let store: Arc<dyn SecretStore> = Arc::new(KeyringStore::new(opts.profile.clone()));
        let mut http = HttpClient::new(base_url.clone(), app_version.clone());
        if let Some(ua) = &opts.user_agent {
            http.set_user_agent(ua.clone()).await;
        }
        if let Some(resolver) = &opts.hv {
            http.set_hv_resolver(resolver.clone());
        }

        let password = SecretString::from(opts.password.clone());
        let login = auth::login(&http, &opts.username, &password, opts.totp.as_deref()).await?;

        let provider = crypto::provider();
        let salts = api::keys::get_key_salts(&http).await?;
        let user = api::keys::get_user(&http).await?;
        let addresses = api::keys::get_addresses(&http).await?;

        let mailbox_pw = if login.password_mode == 2 {
            SecretString::from(opts.mailbox_password.clone().ok_or_else(|| {
                Error::Other(
                    "this account uses a separate mailbox password (PasswordMode 2); supply --mailbox-password".into(),
                )
            })?)
        } else {
            password.clone()
        };

        tracing::debug!(target: "proton_core::mail", addresses = addresses.len(), password_mode = login.password_mode, "login: fetched salts + user + addresses; unlocking keys");
        let (keys, skp) = crypto::keys::unlock(&provider, &user, &addresses, &mailbox_pw, &salts)?;

        let session = Session {
            uid: login.tokens.uid.clone(),
            app_version,
            base_url: base_url.clone(),
            password_mode: login.password_mode,
            user_agent: opts.user_agent.clone(),
        };
        session.save(
            &Paths::system()?,
            &opts.profile,
            store.as_ref(),
            &login.tokens,
            &skp,
        )?;

        Self::wire_refresh(&mut http, store.clone());
        Ok(Client {
            http,
            keys,
            paths: Paths::system()?,
            profile: opts.profile,
            store,
            sender_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Resume a saved session and unlock keys using the stored `skp`.
    pub async fn resume(profile: &str) -> Result<Client> {
        tracing::info!(target: "proton_core::mail", profile, "resume: loading saved session");
        let store: Arc<dyn SecretStore> = Arc::new(KeyringStore::new(profile.to_string()));
        let paths = Paths::system()?;
        let loaded = Session::load(&paths, profile, store.as_ref())?.ok_or(Error::Unauthorized)?;
        tracing::debug!(target: "proton_core::mail", uid = %loaded.session.uid, "resume: session found; fetching user + addresses, unlocking keys");

        let mut http = HttpClient::new(
            loaded.session.base_url.clone(),
            loaded.session.app_version.clone(),
        );
        if let Some(ua) = &loaded.session.user_agent {
            http.set_user_agent(ua.clone()).await;
        }
        let Tokens {
            uid,
            access,
            refresh,
        } = loaded.tokens;
        http.set_tokens(uid, access, refresh).await;
        Self::wire_refresh(&mut http, store.clone());

        let provider = crypto::provider();
        let user = api::keys::get_user(&http).await?;
        let addresses = api::keys::get_addresses(&http).await?;
        let keys = crypto::keys::unlock_with_skp(&provider, &user, &addresses, &loaded.skp)?;

        Ok(Client {
            http,
            keys,
            paths,
            profile: profile.to_string(),
            store,
            sender_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Revoke the server session and clear local state.
    pub async fn logout(&self) -> Result<()> {
        let _ = auth::logout(&self.http).await;
        Session::clear(&self.paths, &self.profile, self.store.as_ref())?;
        Ok(())
    }

    /// The primary sending address email.
    pub fn primary_email(&self) -> Option<&str> {
        self.keys.primary_address().map(|a| a.email.as_str())
    }

    /// Fetch (and cache) a sender's armored public keys for verification.
    pub(crate) async fn sender_pubkeys(&self, email: &str) -> Vec<String> {
        let key = email.to_ascii_lowercase();
        if let Some(v) = self.sender_cache.lock().await.get(&key) {
            return v.clone();
        }
        let pubs = match api::keys::get_all_public_keys(&self.http, email).await {
            Ok(r) => r.keys.into_iter().map(|k| k.public_key).collect(),
            Err(_) => Vec::new(),
        };
        self.sender_cache.lock().await.insert(key, pubs.clone());
        pubs
    }
}
