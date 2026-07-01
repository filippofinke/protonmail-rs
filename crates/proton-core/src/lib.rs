//! `proton-core` — an unofficial, pure-Rust **Proton Mail** client library.
//!
//! End-to-end OpenPGP encryption and SRP authentication, byte-compatible with
//! the official Proton clients, built on Proton's own
//! [proton-crypto](https://github.com/ProtonMail/proton-crypto-rs) and
//! `proton-srp` crates. This is the SDK behind the `protonmail-cli` command-line
//! tool and the `proton-mcp` MCP server.
//!
//! # Overview
//!
//! [`Client`] is the entry point. A session is created out-of-band by
//! `protonmail-cli login` and resumed here with [`Client::resume`]; secrets (auth
//! tokens and the key-unlock passphrase) live in the OS keychain and are never
//! written to disk in cleartext.
//!
//! Capabilities:
//! - **Authentication** — SRP-6a login, TOTP two-factor, two-password accounts,
//!   and a pluggable human-verification (CAPTCHA) hook.
//! - **Crypto** — key-hierarchy unlock, encrypt/decrypt, and signature
//!   verification (the result is a [`Verdict`]).
//! - **Mail** — list/read (decrypt + verify), threads, search, send/reply/
//!   forward (including PGP-to-external and encrypted-outside), attachments,
//!   drafts, Sieve filters, contacts, addresses, settings, and organize
//!   (move/trash/delete/label/star/spam/snooze).
//! - **Local** — event-sync into a SQLite cache and encrypted full-text search.
//!
//! Sending defaults to the primary address (the first alias by API `Order`);
//! pass a `from` address to [`SendOptions`] to use a different alias.
//!
//! # Modules
//!
//! - [`mail`] — the high-level [`Client`] and its mail services.
//! - [`crypto`] — key unlock, encrypt/decrypt, signature verification.
//! - [`api`] — typed wrappers over the Proton HTTP API.
//! - [`transport`] — the HTTP client, auth-token refresh, and request builder.
//! - [`model`] — wire types ([`Message`], [`Conversation`], [`Label`], …).
//! - [`session`] — on-disk session metadata + keychain secret store.
//! - [`cache`] — the local SQLite cache and FTS index.
//! - [`error`] — the crate [`Error`] and [`Result`] alias.
//! - [`html`] — sanitization of HTML message bodies.
//!
//! # Example
//!
//! Resume a session created by `protonmail-cli login` and print the default
//! sending address:
//!
//! ```no_run
//! # async fn demo() -> proton_core::Result<()> {
//! use proton_core::Client;
//!
//! let client = Client::resume("default").await?;
//! println!("default sending address: {:?}", client.primary_email());
//! # Ok(())
//! # }
//! ```
//!
//! > **Unofficial** — not affiliated with or endorsed by Proton AG. Use at your
//! > own risk.
#![warn(missing_docs)]

pub mod api;
pub mod auth;
pub mod cache;
pub mod crypto;
pub mod error;
pub mod html;
pub mod mail;
pub mod model;
pub mod session;
pub mod transport;

pub use api::contacts::{Contact, ContactEmail};
pub use api::events::LabelCount;
pub use api::filters::Filter;
pub use crypto::Verdict;
pub use error::{ApiError, Error, HvChallenge, Result};
pub use mail::contacts::AddressInfo;
pub use mail::read::{FullMessage, SearchOpts};
pub use mail::send::SendOptions;
pub use mail::sync::SyncReport;
pub use mail::{Client, LoginOptions};
pub use model::{Attachment, Conversation, Label, Message, MessageMetadata, Recipient};
pub use transport::{AuthState, Body, Doer, HttpClient, HvResolver, Request, Response};

/// The crate's package version (from `CARGO_PKG_VERSION`).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Initialize stderr tracing. `verbose`: 0=warn, 1=debug (our crates),
/// 2=trace (core), 3+=trace (everything). `RUST_LOG`, if set, overrides.
pub fn init_tracing(verbose: u8) {
    use tracing_subscriber::EnvFilter;
    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        let spec = match verbose {
            0 => "warn",
            1 => "info,proton_core=debug,protonmail_cli=debug,proton_mcp=debug",
            2 => "info,proton_core=trace,protonmail_cli=debug,proton_mcp=debug",
            _ => "trace",
        };
        EnvFilter::new(spec)
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .try_init();
}
