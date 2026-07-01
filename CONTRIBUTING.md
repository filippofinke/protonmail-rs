# Contributing

Thanks for your interest in `protonmail-rs`.

## Ground rules

- This is an **unofficial** client, not affiliated with Proton AG. Keep changes
  compatible with the official wire protocol (SRP, OpenPGP) — do not weaken
  crypto or signature verification.
- Never commit credentials, real session files, or message contents. `.env` and
  `*.session.json` are git-ignored; keep it that way.

## Before opening a PR

Run the same checks CI runs:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

Add tests for new behavior. Live tests that hit a real account are opt-in via
environment variables (see the Testing section in the [README](README.md)) and
must never be required to pass CI.

## Commits & releases

Use [Conventional Commits](https://www.conventionalcommits.org) — they drive
automated versioning and the changelog via release-plz:

- `feat:` → minor bump, `fix:` → patch bump, `feat!:` / `BREAKING CHANGE:` →
  major (pre-1.0: minor).
- `docs:`, `chore:`, `refactor:`, `test:`, `ci:` → no release on their own.

You don't bump versions or edit `CHANGELOG.md` by hand — release-plz opens a
`chore: release` PR that does both. See [RELEASING.md](RELEASING.md).

## Layout

- `crates/proton-core` — the SDK (transport, auth, crypto, mail API). Most logic
  lives here and should be unit-tested.
- `crates/protonmail-cli` — the clap command-line front-end.
- `crates/proton-mcp` — the MCP server; one file per tool under `src/tools/`,
  summed into the router in `src/server.rs`.

## Security

For vulnerabilities, follow [SECURITY.md](SECURITY.md) — do not open a public
issue.
