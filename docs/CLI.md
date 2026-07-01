# protonmail-cli — full command reference

Complete help for every command and flag, generated from `protonmail-cli --help`
by `scripts/gen-cli-docs.sh`. For a quick overview see the [README](../README.md).

- Every command also accepts the [global options](#protonmail-cli) (`--profile`, `--json`, `-v/-vv/-vvv`, `--client`, …); clap repeats them in each command's help.
- `<REF>` = an exact message id **or** free text that uniquely matches one message.

## protonmail-cli

```text
Proton Mail command-line client

Usage: protonmail-cli [OPTIONS] <COMMAND>

Commands:
  login          Log in to Proton Mail (prompts for credentials unless given via env)
  logout         Revoke the server session and clear local state
  whoami         Print the primary email address of the active session
  messages       Message operations
  conversations  Conversation (thread) operations
  attachments    Attachment operations
  drafts         Draft operations (create/edit/delete without sending)
  filters        Server-side Sieve filter operations
  contacts       Contacts (read)
  addresses      Address operations
  settings       Mail settings (read + a few toggles)
  counts         Per-folder message (or conversation) counts
  export         Export messages from a folder to `.eml` files
  watch          Continuously sync the cache (event stream) until interrupted
  index          Build the local encrypted-search index (decrypts + indexes message bodies)
  search         Full-text search the local index (message bodies; private + offline)
  sync           Sync the local cache from the event stream (optionally backfill a folder)
  labels         Label and folder operations
  help           Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
  -V, --version
          Print version
```

## protonmail-cli login

```text
Log in to Proton Mail (prompts for credentials unless given via env)

Usage: protonmail-cli login [OPTIONS]

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli logout

```text
Revoke the server session and clear local state

Usage: protonmail-cli logout [OPTIONS]

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli whoami

```text
Print the primary email address of the active session

Usage: protonmail-cli whoami [OPTIONS]

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli counts

```text
Per-folder message (or conversation) counts

Usage: protonmail-cli counts [OPTIONS]

Options:
      --conversations
          Count conversations instead of messages
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli export

```text
Export messages from a folder to `.eml` files

Usage: protonmail-cli export [OPTIONS] --out <OUT>

Options:
      --folder <FOLDER>
          [default: all]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --out <OUT>
          Output directory
      --max <MAX>
          [default: 100]
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli watch

```text
Continuously sync the cache (event stream) until interrupted

Usage: protonmail-cli watch [OPTIONS]

Options:
      --interval <INTERVAL>
          [default: 30]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --folder <FOLDER>
          Also backfill + index this folder each tick
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli index

```text
Build the local encrypted-search index (decrypts + indexes message bodies)

Usage: protonmail-cli index [OPTIONS]

Options:
      --folder <FOLDER>
          [default: all]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --max-pages <MAX_PAGES>
          [default: 3]
      --page-size <PAGE_SIZE>
          [default: 50]
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli search

```text
Full-text search the local index (message bodies; private + offline)

Usage: protonmail-cli search [OPTIONS] <QUERY>

Arguments:
  <QUERY>  Query words (AND-ed together)

Options:
      --limit <LIMIT>
          [default: 25]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli sync

```text
Sync the local cache from the event stream (optionally backfill a folder)

Usage: protonmail-cli sync [OPTIONS]

Options:
      --backfill <BACKFILL>
          Also backfill this folder into the cache (e.g. `inbox`)
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --max-pages <MAX_PAGES>
          [default: 4]
      --page-size <PAGE_SIZE>
          [default: 50]
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli messages

```text
Message operations

Usage: protonmail-cli messages [OPTIONS] <COMMAND>

Commands:
  list         List messages in a folder
  search       Search messages
  read         Read a single message
  send         Compose and send a message
  reply        Reply to a message
  forward      Forward a message
  cancel-send  Cancel a scheduled send
  spam         Report messages as spam (move to Spam)
  ham          Report messages as not-spam (move to Inbox)
  unsubscribe  One-click unsubscribe from a mailing list
  empty        Permanently empty a folder (e.g. `trash`, `spam`)
  label        Apply or remove a label on messages
  undelete     Restore permanently-deleted messages (by ID)
  receipt      Send a read receipt for a message
  move         Move messages to a folder
  trash        Move messages to Trash
  delete       Permanently delete messages
  mark         Mark messages read or unread
  star         Star messages
  unstar       Unstar messages
  help         Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages list

```text
List messages in a folder

Usage: protonmail-cli messages list [OPTIONS]

Options:
      --folder <FOLDER>
          [default: inbox]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --page <PAGE>
          [default: 0]
      --page-size <PAGE_SIZE>
          [default: 25]
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --unread
          
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --cached
          Read from the local cache instead of the API (offline)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages search

```text
Search messages

Usage: protonmail-cli messages search [OPTIONS]

Options:
      --keyword <KEYWORD>
          Free-text keyword
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --from <FROM>
          Match the sender address
      --json
          Emit machine-readable JSON instead of human-readable text
      --to <TO>
          Match a recipient address
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --subject <SUBJECT>
          Match the subject
      --after <AFTER>
          Only results on/after this date (YYYY-MM-DD)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --before <BEFORE>
          Only results on/before this date (YYYY-MM-DD)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --folder <FOLDER>
          Restrict to a folder/label
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --unread
          Only unread results
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --limit <LIMIT>
          Maximum number of results
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages read

```text
Read a single message

Usage: protonmail-cli messages read [OPTIONS] <REFERENCE>

Arguments:
  <REFERENCE>
          Message reference (ID or free-text)

Options:
      --format <FORMAT>
          Possible values:
          - text: Plain text with a header block
          - html: HTML body with a header block
          - raw:  Raw decrypted body only
          
          [default: text]

      --profile <PROFILE>
          Session profile to use (separates stored credentials)
          
          [default: default]

      --body-only
          Print only the body

      --json
          Emit machine-readable JSON instead of human-readable text

      --output <OUTPUT>
          Write the body to a file instead of stdout

  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call

      --api-url <API_URL>
          Override the API base URL (login only)

      --app-version <APP_VERSION>
          Override the app-version header (login only)

      --totp <TOTP>
          TOTP two-factor code
          
          [env: PROTON_TOTP=]

      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts)
          
          [env: PROTON_MAILBOX_PASSWORD=]

      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow
          
          [env: PROTON_HV_TOKEN=]

      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden)
          
          [possible values: web, ios, android]

      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)

      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified

  -h, --help
          Print help (see a summary with '-h')
```

### protonmail-cli messages send

```text
Compose and send a message

Usage: protonmail-cli messages send [OPTIONS] --to <TO> --subject <SUBJECT> --body <BODY>

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --to <TO>
          Primary recipient(s) (repeatable)
      --cc <CC>
          Carbon-copy recipient(s) (repeatable)
      --json
          Emit machine-readable JSON instead of human-readable text
      --bcc <BCC>
          Blind carbon-copy recipient(s) (repeatable)
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --from <FROM>
          Sender address (defaults to the primary address)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --subject <SUBJECT>
          Message subject
      --body <BODY>
          Message body, or `-` to read from stdin
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --html
          Treat the body as HTML
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --attach <ATTACH>
          Attach a file (repeatable)
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --send-at <SEND_AT>
          Schedule delivery at this Unix timestamp
      --expires <EXPIRES>
          Self-destruct after this many seconds
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
      --eo-password <EO_PASSWORD>
          Encrypted-outside: password-protect the message for keyless external recipients
      --eo-hint <EO_HINT>
          Password hint shown to encrypted-outside recipients
  -h, --help
          Print help
```

### protonmail-cli messages reply

```text
Reply to a message

Usage: protonmail-cli messages reply [OPTIONS] --body <BODY> <REFERENCE>

Arguments:
  <REFERENCE>  

Options:
      --all
          Reply to all recipients
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --from <FROM>
          
      --json
          Emit machine-readable JSON instead of human-readable text
      --body <BODY>
          Body text, or `-` for stdin
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --attach <ATTACH>
          
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages forward

```text
Forward a message

Usage: protonmail-cli messages forward [OPTIONS] --to <TO> --body <BODY> <REFERENCE>

Arguments:
  <REFERENCE>  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --to <TO>
          
      --from <FROM>
          
      --json
          Emit machine-readable JSON instead of human-readable text
      --body <BODY>
          
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --attach <ATTACH>
          
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages cancel-send

```text
Cancel a scheduled send

Usage: protonmail-cli messages cancel-send [OPTIONS] <REFERENCE>

Arguments:
  <REFERENCE>  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages spam

```text
Report messages as spam (move to Spam)

Usage: protonmail-cli messages spam [OPTIONS] [REFERENCES]...

Arguments:
  [REFERENCES]...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages ham

```text
Report messages as not-spam (move to Inbox)

Usage: protonmail-cli messages ham [OPTIONS] [REFERENCES]...

Arguments:
  [REFERENCES]...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages unsubscribe

```text
One-click unsubscribe from a mailing list

Usage: protonmail-cli messages unsubscribe [OPTIONS] <REFERENCE>

Arguments:
  <REFERENCE>  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages empty

```text
Permanently empty a folder (e.g. `trash`, `spam`)

Usage: protonmail-cli messages empty [OPTIONS] <FOLDER>

Arguments:
  <FOLDER>  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages label

```text
Apply or remove a label on messages

Usage: protonmail-cli messages label [OPTIONS] <ACTION> <LABEL_ID> [REFERENCES]...

Arguments:
  <ACTION>         [possible values: add, rm]
  <LABEL_ID>       Label ID
  [REFERENCES]...  Message references

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages undelete

```text
Restore permanently-deleted messages (by ID)

Usage: protonmail-cli messages undelete [OPTIONS] [IDS]...

Arguments:
  [IDS]...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages receipt

```text
Send a read receipt for a message

Usage: protonmail-cli messages receipt [OPTIONS] <REFERENCE>

Arguments:
  <REFERENCE>  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages move

```text
Move messages to a folder

Usage: protonmail-cli messages move [OPTIONS] --dest <DEST> <REFERENCES>...

Arguments:
  <REFERENCES>...  

Options:
      --dest <DEST>
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages trash

```text
Move messages to Trash

Usage: protonmail-cli messages trash [OPTIONS] <REFERENCES>...

Arguments:
  <REFERENCES>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages delete

```text
Permanently delete messages

Usage: protonmail-cli messages delete [OPTIONS] <REFERENCES>...

Arguments:
  <REFERENCES>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages mark

```text
Mark messages read or unread

Usage: protonmail-cli messages mark [OPTIONS] <STATE> <REFERENCES>...

Arguments:
  <STATE>          [possible values: read, unread]
  <REFERENCES>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages star

```text
Star messages

Usage: protonmail-cli messages star [OPTIONS] <REFERENCES>...

Arguments:
  <REFERENCES>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli messages unstar

```text
Unstar messages

Usage: protonmail-cli messages unstar [OPTIONS] <REFERENCES>...

Arguments:
  <REFERENCES>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli conversations

```text
Conversation (thread) operations

Usage: protonmail-cli conversations [OPTIONS] <COMMAND>

Commands:
  list      List conversations in a folder
  search    Search conversations
  read      Read a conversation (all messages)
  move      Move conversations to a folder
  trash     Move conversations to Trash
  mark      Mark conversations read or unread
  star      Star conversations
  unstar    Unstar conversations
  snooze    Snooze conversations until a Unix timestamp (--until) or a duration (--in 1h/2d)
  unsnooze  Unsnooze conversations
  help      Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations list

```text
List conversations in a folder

Usage: protonmail-cli conversations list [OPTIONS]

Options:
      --folder <FOLDER>
          [default: inbox]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --page <PAGE>
          [default: 0]
      --page-size <PAGE_SIZE>
          [default: 25]
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --unread
          
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations search

```text
Search conversations

Usage: protonmail-cli conversations search [OPTIONS]

Options:
      --keyword <KEYWORD>
          Free-text keyword
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --from <FROM>
          Match the sender address
      --json
          Emit machine-readable JSON instead of human-readable text
      --to <TO>
          Match a recipient address
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --subject <SUBJECT>
          Match the subject
      --after <AFTER>
          Only results on/after this date (YYYY-MM-DD)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --before <BEFORE>
          Only results on/before this date (YYYY-MM-DD)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --folder <FOLDER>
          Restrict to a folder/label
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --unread
          Only unread results
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --limit <LIMIT>
          Maximum number of results
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations read

```text
Read a conversation (all messages)

Usage: protonmail-cli conversations read [OPTIONS] <ID>

Arguments:
  <ID>  Conversation ID

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations move

```text
Move conversations to a folder

Usage: protonmail-cli conversations move [OPTIONS] --dest <DEST> <IDS>...

Arguments:
  <IDS>...  

Options:
      --dest <DEST>
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations trash

```text
Move conversations to Trash

Usage: protonmail-cli conversations trash [OPTIONS] <IDS>...

Arguments:
  <IDS>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations mark

```text
Mark conversations read or unread

Usage: protonmail-cli conversations mark [OPTIONS] <STATE> <IDS>...

Arguments:
  <STATE>   [possible values: read, unread]
  <IDS>...  

Options:
      --folder <FOLDER>
          Label context for the mark operation [default: all]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations star

```text
Star conversations

Usage: protonmail-cli conversations star [OPTIONS] <IDS>...

Arguments:
  <IDS>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations unstar

```text
Unstar conversations

Usage: protonmail-cli conversations unstar [OPTIONS] <IDS>...

Arguments:
  <IDS>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations snooze

```text
Snooze conversations until a Unix timestamp (--until) or a duration (--in 1h/2d)

Usage: protonmail-cli conversations snooze [OPTIONS] <IDS>...

Arguments:
  <IDS>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --until <UNTIL>
          
      --in <RELATIVE>
          Relative duration from now, e.g. `30m`, `2h`, `1d`
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli conversations unsnooze

```text
Unsnooze conversations

Usage: protonmail-cli conversations unsnooze [OPTIONS] <IDS>...

Arguments:
  <IDS>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli attachments

```text
Attachment operations

Usage: protonmail-cli attachments [OPTIONS] <COMMAND>

Commands:
  list      List a message's attachments
  download  Download attachment(s) from a message
  help      Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli attachments list

```text
List a message's attachments

Usage: protonmail-cli attachments list [OPTIONS] <MESSAGE>

Arguments:
  <MESSAGE>  Message reference (ID or free-text)

Options:
      --include-inline
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli attachments download

```text
Download attachment(s) from a message

Usage: protonmail-cli attachments download [OPTIONS] <MESSAGE> [ATTACHMENT]

Arguments:
  <MESSAGE>     Message reference (ID or free-text)
  [ATTACHMENT]  Specific attachment ID (omit with --all)

Options:
      --output-dir <OUTPUT_DIR>
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --all
          Download all attachments
      --json
          Emit machine-readable JSON instead of human-readable text
      --include-inline
          
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli drafts

```text
Draft operations (create/edit/delete without sending)

Usage: protonmail-cli drafts [OPTIONS] <COMMAND>

Commands:
  list    List drafts
  save    Save a new draft (does not send)
  edit    Replace an existing draft's content
  delete  Delete drafts
  help    Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli drafts list

```text
List drafts

Usage: protonmail-cli drafts list [OPTIONS]

Options:
      --page <PAGE>
          [default: 0]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --page-size <PAGE_SIZE>
          [default: 25]
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli drafts save

```text
Save a new draft (does not send)

Usage: protonmail-cli drafts save [OPTIONS] --to <TO> --subject <SUBJECT> --body <BODY>

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --to <TO>
          Primary recipient(s) (repeatable)
      --cc <CC>
          Carbon-copy recipient(s) (repeatable)
      --json
          Emit machine-readable JSON instead of human-readable text
      --bcc <BCC>
          Blind carbon-copy recipient(s) (repeatable)
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --from <FROM>
          Sender address (defaults to the primary address)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --subject <SUBJECT>
          Message subject
      --body <BODY>
          Message body, or `-` to read from stdin
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --html
          Treat the body as HTML
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --attach <ATTACH>
          Attach a file (repeatable)
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --send-at <SEND_AT>
          Schedule delivery at this Unix timestamp
      --expires <EXPIRES>
          Self-destruct after this many seconds
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
      --eo-password <EO_PASSWORD>
          Encrypted-outside: password-protect the message for keyless external recipients
      --eo-hint <EO_HINT>
          Password hint shown to encrypted-outside recipients
  -h, --help
          Print help
```

### protonmail-cli drafts edit

```text
Replace an existing draft's content

Usage: protonmail-cli drafts edit [OPTIONS] --to <TO> --subject <SUBJECT> --body <BODY> <ID>

Arguments:
  <ID>  Draft message ID

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --to <TO>
          Primary recipient(s) (repeatable)
      --cc <CC>
          Carbon-copy recipient(s) (repeatable)
      --json
          Emit machine-readable JSON instead of human-readable text
      --bcc <BCC>
          Blind carbon-copy recipient(s) (repeatable)
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --from <FROM>
          Sender address (defaults to the primary address)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --subject <SUBJECT>
          Message subject
      --body <BODY>
          Message body, or `-` to read from stdin
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --html
          Treat the body as HTML
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --attach <ATTACH>
          Attach a file (repeatable)
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --send-at <SEND_AT>
          Schedule delivery at this Unix timestamp
      --expires <EXPIRES>
          Self-destruct after this many seconds
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
      --eo-password <EO_PASSWORD>
          Encrypted-outside: password-protect the message for keyless external recipients
      --eo-hint <EO_HINT>
          Password hint shown to encrypted-outside recipients
  -h, --help
          Print help
```

### protonmail-cli drafts delete

```text
Delete drafts

Usage: protonmail-cli drafts delete [OPTIONS] [IDS]...

Arguments:
  [IDS]...  Draft message IDs

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli filters

```text
Server-side Sieve filter operations

Usage: protonmail-cli filters [OPTIONS] <COMMAND>

Commands:
  list     List filters
  create   Create a filter from a Sieve script (`--sieve -` reads stdin)
  check    Validate a Sieve script without creating a filter
  delete   Delete a filter
  enable   Enable a filter
  disable  Disable a filter
  help     Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli filters list

```text
List filters

Usage: protonmail-cli filters list [OPTIONS]

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli filters create

```text
Create a filter from a Sieve script (`--sieve -` reads stdin)

Usage: protonmail-cli filters create [OPTIONS] --name <NAME> --sieve <SIEVE>

Options:
      --name <NAME>
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --sieve <SIEVE>
          Sieve script text, or `-` for stdin
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli filters check

```text
Validate a Sieve script without creating a filter

Usage: protonmail-cli filters check [OPTIONS] --sieve <SIEVE>

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --sieve <SIEVE>
          Sieve script text, or `-` for stdin
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli filters delete

```text
Delete a filter

Usage: protonmail-cli filters delete [OPTIONS] <ID>

Arguments:
  <ID>  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli filters enable

```text
Enable a filter

Usage: protonmail-cli filters enable [OPTIONS] <ID>

Arguments:
  <ID>  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli filters disable

```text
Disable a filter

Usage: protonmail-cli filters disable [OPTIONS] <ID>

Arguments:
  <ID>  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli contacts

```text
Contacts (read)

Usage: protonmail-cli contacts [OPTIONS] <COMMAND>

Commands:
  list    List contacts
  emails  List contact email addresses (optionally filter by an email)
  help    Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli contacts list

```text
List contacts

Usage: protonmail-cli contacts list [OPTIONS]

Options:
      --page <PAGE>
          [default: 0]
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --page-size <PAGE_SIZE>
          [default: 50]
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli contacts emails

```text
List contact email addresses (optionally filter by an email)

Usage: protonmail-cli contacts emails [OPTIONS]

Options:
      --email <EMAIL>
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli addresses

```text
Address operations

Usage: protonmail-cli addresses [OPTIONS] <COMMAND>

Commands:
  list    List addresses
  update  Update an address (display name / signature)
  help    Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli addresses list

```text
List addresses

Usage: protonmail-cli addresses list [OPTIONS]

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli addresses update

```text
Update an address (display name / signature)

Usage: protonmail-cli addresses update [OPTIONS] <ID>

Arguments:
  <ID>  

Options:
      --display-name <DISPLAY_NAME>
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
      --signature <SIGNATURE>
          
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli settings

```text
Mail settings (read + a few toggles)

Usage: protonmail-cli settings [OPTIONS] <COMMAND>

Commands:
  get                Show mail settings (raw JSON)
  sign               Toggle signing outgoing mail
  attach-public-key  Toggle attaching your public key to outgoing mail
  help               Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli settings get

```text
Show mail settings (raw JSON)

Usage: protonmail-cli settings get [OPTIONS]

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli settings sign

```text
Toggle signing outgoing mail

Usage: protonmail-cli settings sign [OPTIONS] <VALUE>

Arguments:
  <VALUE>  [possible values: on, off]

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli settings attach-public-key

```text
Toggle attaching your public key to outgoing mail

Usage: protonmail-cli settings attach-public-key [OPTIONS] <VALUE>

Arguments:
  <VALUE>  [possible values: on, off]

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

## protonmail-cli labels

```text
Label and folder operations

Usage: protonmail-cli labels [OPTIONS] <COMMAND>

Commands:
  list    List labels (or folders with --folders)
  create  Create a label or folder
  delete  Delete labels/folders by ID
  update  Rename / recolor / reparent a label or folder
  help    Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli labels list

```text
List labels (or folders with --folders)

Usage: protonmail-cli labels list [OPTIONS]

Options:
      --folders
          List folders instead of labels
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli labels create

```text
Create a label or folder

Usage: protonmail-cli labels create [OPTIONS] --name <NAME>

Options:
      --name <NAME>
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --color <COLOR>
          [default: #8080FF]
      --json
          Emit machine-readable JSON instead of human-readable text
      --folder
          Create a folder instead of a label
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --parent <PARENT>
          
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli labels delete

```text
Delete labels/folders by ID

Usage: protonmail-cli labels delete [OPTIONS] <IDS>...

Arguments:
  <IDS>...  

Options:
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --json
          Emit machine-readable JSON instead of human-readable text
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

### protonmail-cli labels update

```text
Rename / recolor / reparent a label or folder

Usage: protonmail-cli labels update [OPTIONS] <ID>

Arguments:
  <ID>  

Options:
      --name <NAME>
          
      --profile <PROFILE>
          Session profile to use (separates stored credentials) [default: default]
      --color <COLOR>
          
      --json
          Emit machine-readable JSON instead of human-readable text
      --parent <PARENT>
          
  -v, --verbose...
          Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all). `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call
      --api-url <API_URL>
          Override the API base URL (login only)
      --app-version <APP_VERSION>
          Override the app-version header (login only)
      --totp <TOTP>
          TOTP two-factor code [env: PROTON_TOTP=]
      --mailbox-password <MAILBOX_PASSWORD>
          Separate mailbox password (two-password accounts) [env: PROTON_MAILBOX_PASSWORD=]
      --captcha-token <CAPTCHA_TOKEN>
          Pre-solved human-verification (CAPTCHA) token, skips the interactive flow [env: PROTON_HV_TOKEN=]
      --client <CLIENT>
          Present as an official client (sets app-version + User-Agent unless overridden) [possible values: web, ios, android]
      --user-agent <USER_AGENT>
          Override the User-Agent header (login only)
      --captcha-chrome
          Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead of your default browser; it is closed automatically once verified
  -h, --help
          Print help
```

