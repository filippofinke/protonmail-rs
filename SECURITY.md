# Security Policy

This project is an **unofficial** Proton Mail client. It handles credentials,
OpenPGP private keys, and plaintext message bodies, so security reports are
taken seriously.

## Reporting a vulnerability

**Do not open a public issue for security problems.**

Report privately via GitHub's
[private vulnerability reporting](https://github.com/filippofinke/protonmail-rs/security/advisories/new),
or by email to **filippofinke@protonmail.ch**.

Please include a description, affected version/commit, and reproduction steps.
You will get an acknowledgement within a reasonable timeframe and an update once
the issue is assessed.

## Scope

In scope: anything that could expose credentials, private keys, decrypted
plaintext, or session tokens; signature-verification bypasses; weakening of the
SRP/OpenPGP handling.

Out of scope: issues in upstream dependencies (`proton-crypto`, `proton-srp`,
`rustls`, …) — report those upstream — and the Proton API itself.

## Handling notes (how secrets are treated)

- The mailbox password is never written to disk in cleartext. The unlocking key
  passphrase and auth tokens are stored in the OS keychain; only non-secret
  session metadata is written to disk (mode `0600`).
- Address-key token signatures are verified on unlock; the SRP server proof and
  the signed modulus are verified on login.
- Verbose logs are designed to exclude secrets and plaintext. If you find a log
  line that leaks either, that is a vulnerability — please report it.

This software is provided "as is", without warranty (see [LICENSE](LICENSE)).
Use at your own risk.
