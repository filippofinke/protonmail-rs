<h1 align="center">Welcome to protonmail-rs 👋</h1>

> An unofficial, pure-Rust Proton Mail client — library, CLI, and MCP server.

An unofficial, pure-Rust **Proton Mail** client: a library (`proton-core`), a
command-line tool (`protonmail-cli`), and an MCP server (`proton-mcp`). End-to-end
OpenPGP encryption and SRP authentication are byte-compatible with the official
Proton clients, built on Proton's own Rust crates
([`proton-crypto`](https://github.com/ProtonMail/proton-crypto-rs), `proton-srp`).

> Unofficial, not affiliated with Proton AG. Use at your own risk.

Validated against a live account: SRP login (incl. CAPTCHA), session resume, key
unlock, **send** (E2E), **list**, **read + decrypt + signature verify** all work.

---

## Workspace

| Crate (crates.io) | Binary | Role |
|---|---|---|
| `proton-core` | — | The SDK — transport, auth, crypto, models, API, mail services |
| `protonmail-cli` | `protonmail-cli` | Command-line front-end (clap) |
| `proton-mcp` | `proton-mcp` | MCP server exposing mail tools to LLM agents (rmcp) |

Requires Rust **≥ 1.96**.

```bash
# Install from crates.io:
cargo install protonmail-cli      # -> protonmail-cli binary
cargo install proton-mcp          # -> proton-mcp

# Or build from source:
cargo build --release             # -> target/release/{protonmail-cli, proton-mcp}
```

---

## Feature list — `proton-core` (the SDK)

### Authentication
- **SRP login** (SRP-6a) via `proton-srp` — server-signed modulus verified, client/server proof exchange, hash versions V0–V4.
- **Two-factor (TOTP)**.
- **Two-password accounts** — reads `PasswordMode`; separate mailbox password supported.
- **Human verification / CAPTCHA** — pluggable resolver on API code 9001 (retries with `x-pm-human-verification-token`).
- **Session persistence** — per-profile, multi-profile; non-secret metadata on disk (0600), secrets (tokens + key passphrase) in the **OS keychain** (`keyring`).
- **Session resume** — re-open + unlock keys from a saved session without re-entering the password.
- **Token refresh** — automatic on 401 (rotating tokens persisted), single retry.
- **Logout** — server-side session revoke + local clear.
- **Client masquerade** — configurable `x-pm-appversion` + `User-Agent` (web/iOS/Android presets).

### Cryptography (pure Rust, no cgo)
- **Key hierarchy unlock** — user keys (bcrypt mailbox-password KDF → key passphrase) and address keys (per-key token decrypted + **signature verified** against user keys; bad tokens skipped).
- **Mailbox-password KDF** — bcrypt, last-31-byte convention, salt matched to the primary user key.
- **Message body decryption** with **signature verdict** (`Verified` / `Unsigned` / `Unverified` / `Invalid`); non-fatal signature failures.
- **MIME extraction** — `multipart/*` bodies parsed (`mail-parser`) to the displayable text/html part.
- **Attachment decryption** — key packet → session key → data packet.
- **Outbound encryption** — fresh AES-256 session key, encrypt+sign body; per-recipient session-key wrapping; self-encrypted stored draft.
- **Attachment encryption** — session key + data packet + detached signature; re-keying for forwards.

### Transport
- `reqwest` (rustls TLS, cookie store); base URL + app-version configurable.
- Proton envelope handling (`Code` 1000/1001 = success).
- Auto-retry: **401 → refresh**, **429 → Retry-After backoff**, **9001 → human verification**.
- Typed errors with CLI exit-code mapping.

### Mail features
- **Read**: list messages, list conversations (threads), read message (decrypt + verdict), read full thread (lazy-loads + decrypts each), counts.
- **Search** (server-side): keyword, from, to, subject, date range, folder, unread.
- **Send**: compose, **reply / reply-all / forward** (forward re-keys original attachments), **scheduled send**, **expiring/self-destruct**, **cancel scheduled send**. Internal = full **E2E** (Type 1); **external with a PGP/WKD key = PGP-inline (Type 8)**; keyless external = cleartext (Type 4, warned).
- **Attachments**: list, download (decrypt) single/all, upload (encrypt) on send.
- **Organize**: move to folder, trash, permanent delete, mark read/unread, star/unstar, apply/remove label — for **messages and conversations**.
- **Labels / folders**: list, create, delete.
- **Drafts**: save / edit / list / delete (without sending).
- **Sieve filters**: list / create (raw Sieve) / validate / delete / enable / disable.
- **Contacts** (read): list contacts + contact emails; **addresses** listing.
- **Local cache + sync**: SQLite cache fed by the Proton **event stream** (`sync`); offline cached reads (`list --cached`).
- **Encrypted full-text search**: build a local **FTS5** index over decrypted bodies (`index`) and search them privately/offline (`search`) — the server never sees the query.

### Observability
- **Verbose logging** (`tracing`) — `init_tracing(verbose)`; logs every login stage, key-unlock step, encrypt/decrypt op, send-pipeline stage, and HTTP request/response. Secrets and plaintext are never logged.

---

## `protonmail-cli` — commands

Every command and flag is documented in **[docs/CLI.md](docs/CLI.md)** (generated
from `--help`). Cheat-sheet of the full surface:

```
protonmail-cli [GLOBALS] <command>

GLOBALS: --profile --json -v/-vv/-vvv --api-url --app-version --client web|ios|android
         --user-agent --totp --mailbox-password --captcha-token --captcha-chrome
ENV:     PROTON_USER PROTON_PASSWORD PROTON_TOTP PROTON_MAILBOX_PASSWORD PROTON_HV_TOKEN  RUST_LOG
COMPOSE (send/reply/forward/drafts save|edit): --to --cc --bcc --from --subject
         --body <text|-> --html --attach <file> --send-at <ts> --expires <secs>
         --eo-password --eo-hint        # send only: encrypted-outside for keyless recipients

login | logout | whoami
messages       list [--folder inbox] [--page --page-size --unread --cached] | search [filters] |
               read <REF> [--format text|html|raw] [--body-only] [--output <file>] |
               send <COMPOSE> | reply <REF> [--all] | forward <REF> --to | cancel-send <REF> |
               spam <REF…> | ham <REF…> | unsubscribe <REF> | empty <folder> | undelete <ID…> |
               receipt <REF> | label <add|rm> <LABEL_ID> <REF…> |
               move <REF…> --dest | trash <REF…> | delete <REF…> |
               mark <read|unread> <REF…> | star <REF…> | unstar <REF…>
conversations  list | search | read <ID> | move <ID…> --dest | trash <ID…> |
               mark <read|unread> <ID…> | star | unstar |
               snooze <ID…> (--until <ts> | --in 30m|2h|1d) | unsnooze <ID…>
attachments    list <MSG> [--include-inline] | download <MSG> [<ATT>] [--output-dir|--all|--include-inline]
drafts         list | save <COMPOSE> | edit <ID> <COMPOSE> | delete <ID…>
filters        list | create --name --sieve <-|text> | check --sieve | delete|enable|disable <ID>
labels         list [--folders] | create --name [--color --folder --parent] | delete <ID…> |
               update <ID> [--name --color --parent]
contacts       list | emails [--email]
addresses      list | update <ID> [--display-name --signature]
settings       get | sign <on|off> | attach-public-key <on|off>
counts         [--conversations]
export         --out <dir> [--folder all] [--max 100]                # write messages to .eml files
watch          [--interval 30] [--folder <f>]                        # continuous cache sync until ^C
sync           [--backfill <folder>]                                 # one-shot event-stream sync
index          [--folder all] [--max-pages --page-size]              # build local encrypted FTS index
search         <query> [--limit 25]                                  # query the local FTS index (offline)
```
`<REF>` = exact message ID or free text (unique search match); `<ID>` = exact id. `--json` on any
command for machine output. CAPTCHA: login auto-opens it; solve via console snippet / bookmarklet,
or pass `--captcha-token`, or `--captcha-chrome` (isolated, auto-closing window).

---

## `proton-mcp` — MCP server (for LLM agents)

Reuses a `protonmail-cli login` session (the agent never sees the password). Transports: **stdio** (default) and **Streamable HTTP** (`--http <addr>`). Verbose via `-v`. Source is **one file per tool** under `src/tools/`.

**56 tools** — full parity with the CLI (auth/`watch`/`export` aside):

- **Messages — read:** `list_messages`, `search_messages`, `read_message`, `list_attachments`, `download_attachment`, `list_labels`, `counts`.
- **Messages — write:** `send_message`, `reply_message`, `forward_message`, `cancel_scheduled_send`, `move_message`, `trash_message`, `delete_messages`, `undelete_messages`, `empty_folder`, `report_spam`, `report_ham`, `unsubscribe`, `send_read_receipt`, `mark_read`, `star_message`, `apply_label`, `remove_label`.
- **Conversations:** `list_conversations`, `read_conversation`, `search_conversations`, `move_conversation`, `trash_conversation`, `mark_conversation`, `star_conversation`, `snooze_conversation`, `unsnooze_conversation`.
- **Drafts:** `list_drafts`, `save_draft`, `update_draft`, `delete_draft`.
- **Filters (Sieve):** `list_filters`, `create_filter`, `check_filter`, `delete_filter`, `enable_filter`, `disable_filter`.
- **Labels/folders:** `create_label`, `delete_label`, `update_label`.
- **Settings / addresses / contacts:** `get_settings`, `set_sign`, `set_attach_public_key`, `list_addresses`, `update_address`, `list_contacts`, `list_contact_emails`.
- **Local cache / search:** `sync_cache`, `index_cache`, `search_local`.

**Safety:** destructive/structural tools (send, move, trash, **delete**, **empty_folder**, drafts, filters, labels, settings, …) take `confirm` and return a **dry-run preview** unless `confirm: true` (or the server is started with `--allow-writes`); `empty_folder`/`delete_messages` previews carry an explicit irreversibility warning. Benign toggles (`mark_*`, `star_*`, `snooze_*`) and all reads execute directly. Sending defaults to your **primary address** (first alias by `Order`) — pass `from` to pick another alias. Reads return the signature **verdict**; CAPTCHA is never auto-solved (returns "run `protonmail-cli login`").

**Claude Desktop config:**
```json
{ "mcpServers": { "proton-mail": {
  "command": "/abs/path/protonmail-rs/target/release/proton-mcp",
  "args": ["--profile","default"] } } }
```

---

## Verbose / debugging
```bash
protonmail-cli -vv whoami                 # login/resume + key unlock, step by step
protonmail-cli -vv messages read <ID>     # + decryption + signature verdict
protonmail-cli -vv messages send …        # + full send pipeline
RUST_LOG=proton_core::http=debug …    # just HTTP
proton-mcp -vv --profile default      # MCP verbose (to stderr)
```

## Testing
```bash
cargo test --workspace                # offline: crypto round-trips, transport, HV, api, cli, mcp
# live (real account; opt-in):
PROTON_TEST_USER=… PROTON_TEST_PASSWORD=… [PROTON_TEST_TOTP=…] [PROTON_TEST_SEND=1] \
  cargo test -p proton-core --test live -- --nocapture --test-threads=1
```

## Security
- Mailbox password never written to disk in cleartext; key passphrase + tokens in the OS keychain.
- Address-key token signatures verified on unlock; SRP server proof + signed modulus verified on login.
- TLS via rustls. Verbose logs exclude all secrets and plaintext.

## Scope / deferred
Implemented: full mail (read/threads, search, send/reply/forward incl. **PGP-external** and **encrypted-outside**, attachments, organize), **drafts**, **Sieve filters**, **mail settings (read) + spam/ham/unsubscribe/empty-folder/snooze**, **contacts (read) + addresses**, **event-sync + SQLite cache**, **local encrypted full-text search**, **HTML sanitization** of message bodies, CLI, MCP, CAPTCHA, verbose logging.

**Not implemented (deliberately, to avoid shipping untested code):**
- **Key Transparency** (epoch / VRF / certificate-transparency chain) — `proton-crypto` exposes only result types; the full verification is a large standalone effort. Recipient keys are trusted as the server returns them.
- **FIDO2 / WebAuthn 2FA** — needs a hardware key + a CTAP client + live testing; login **detects** FIDO2-only accounts and returns a clear error. Use TOTP or an app/bridge password.
- Remote-image proxy, key/address management (generate/rotate), auto-reply/vacation, Drive / Calendar / Pass.

---

## Contributing

Issues and PRs welcome — see [CONTRIBUTING.md](CONTRIBUTING.md). Run
`cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
and `cargo test --workspace` before opening a PR. Release process:
[RELEASING.md](RELEASING.md).

## Security

This client handles credentials, private keys, and plaintext mail. Report
vulnerabilities privately per [SECURITY.md](SECURITY.md) — do not open a public
issue.

## License

[MIT](LICENSE) © Filippo Finke.

> **Disclaimer:** Unofficial, not affiliated with or endorsed by Proton AG.
> "Proton", "Proton Mail" and related marks belong to Proton AG. Use at your own
> risk.

## Author

👤 **Filippo Finke**

- Website: https://filippofinke.ch
- Github: [@filippofinke](https://github.com/filippofinke)
- LinkedIn: [@filippofinke](https://linkedin.com/in/filippofinke)

## Show your support

Give a ⭐️ if this project helped you!

<a href="https://www.buymeacoffee.com/filippofinke">
  <img src="https://github.com/filippofinke/filippofinke/raw/main/images/buymeacoffe.png" alt="Buy Me A McFlurry">
</a>
