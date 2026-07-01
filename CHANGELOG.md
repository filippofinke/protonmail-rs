# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-30

Initial release. Unofficial, pure-Rust Proton Mail client.

### Added
- **`proton-core`** — the SDK:
  - SRP-6a login (server-signed modulus verified, hash versions V0–V4), TOTP
    two-factor, two-password accounts, human-verification/CAPTCHA hook.
  - Session persistence (non-secret metadata on disk `0600`; tokens + key
    passphrase in the OS keychain) and resume without re-entering the password.
  - OpenPGP end-to-end crypto via Proton's `proton-crypto`/`proton-srp`:
    key-hierarchy unlock, encrypt/decrypt, signature verification.
  - Mail: list/read (decrypt + verify), threads, search, send/reply/forward
    (incl. PGP-to-external and encrypted-outside), attachments, organize
    (move/trash/delete/label/star/spam/ham/snooze), drafts, Sieve filters,
    contacts (read), addresses, mail settings, per-folder counts.
  - Event-sync + SQLite local cache and local encrypted full-text search (FTS5).
  - HTML sanitization of message bodies.
- **`protonmail-cli`** — command-line client over `proton-core` (installs the
  `protonmail-cli` binary).
- **`proton-mcp`** — Model Context Protocol server exposing 56 mail tools to LLM
  agents; destructive tools are confirm-gated (dry-run preview by default).

### Notes
- Sending defaults to the primary address (first alias by API `Order`); pass
  `--from` / `from` to use a different alias.
- Key Transparency and FIDO2/U2F second factors are detected but not yet
  implemented.

[Unreleased]: https://github.com/filippofinke/protonmail-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/filippofinke/protonmail-rs/releases/tag/v0.1.0
