# Releasing

Releases are automated with [release-plz](https://release-plz.dev) — see
[`release-plz.toml`](release-plz.toml) and
[`.github/workflows/release-plz.yml`](.github/workflows/release-plz.yml).

The workspace publishes three crates in dependency order (release-plz resolves
this automatically):

```
proton-core  ←  protonmail-cli
             ←  proton-mcp
```

All three share one version (`version.workspace = true`) and are bumped and
released together.

## One-time setup

1. Create a crates.io API token (scopes: `publish-new`, `publish-update`) and
   add it as the repository secret **`CARGO_REGISTRY_TOKEN`**.
2. (Optional, for CI to run on the release PR) Create a fine-grained PAT with
   `contents` + `pull-requests` write, add it as **`RELEASE_PLZ_TOKEN`**, and
   pass it to the release-plz action via `token:` in the workflow.

## Automated flow

1. Merge conventional-commit PRs into `main` (`feat:`, `fix:`, `feat!:`/`BREAKING
   CHANGE:` for majors, etc.).
2. The **release-pr** job opens/updates a `chore: release` PR that bumps the
   workspace version and regenerates `CHANGELOG.md` from those commits.
3. Merge that PR. The **release** job then runs tests, publishes the crates to
   crates.io (core first), and creates the `vX.Y.Z` git tag + GitHub release.

The very first push to `main` publishes `0.1.0` directly (the crates are not yet
on crates.io, so there is nothing to diff against).

## Manual fallback

If publishing by hand (e.g. release-plz is unavailable):

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace

cargo publish -p proton-core      # must be first
# wait for the crates.io index to refresh, then:
cargo publish -p protonmail-cli
cargo publish -p proton-mcp

git tag v0.1.0 && git push origin main --tags
```

Until `proton-core` exists on crates.io, packaging the binaries fails with "no
matching package named `proton-core` found" — expected; publish core first.

## Notes

- Conventional commit prefixes drive the version bump and changelog, so keep
  commit subjects accurate (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, …).
- release-plz runs `cargo-semver-checks` on `proton-core` (the library) to catch
  accidental API breaks and pick the right bump.
- `cargo package -p proton-core` should report zero warnings before a release.
