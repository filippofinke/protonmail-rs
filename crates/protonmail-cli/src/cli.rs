//! Command-line interface definition (clap v4 derive).

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Proton Mail command-line client.
#[derive(Debug, Parser)]
#[command(
    name = "protonmail-cli",
    version,
    about = "Proton Mail command-line client"
)]
pub struct Cli {
    /// Session profile to use (separates stored credentials).
    #[arg(long, global = true, default_value = "default")]
    pub profile: String,

    /// Emit machine-readable JSON instead of human-readable text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all).
    /// `RUST_LOG` overrides. Logs steps of login, crypto, and every HTTP call.
    #[arg(short = 'v', long = "verbose", global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Override the API base URL (login only).
    #[arg(long, global = true)]
    pub api_url: Option<String>,

    /// Override the app-version header (login only).
    #[arg(long, global = true)]
    pub app_version: Option<String>,

    /// TOTP two-factor code.
    #[arg(long, global = true, env = "PROTON_TOTP")]
    pub totp: Option<String>,

    /// Separate mailbox password (two-password accounts).
    #[arg(long, global = true, env = "PROTON_MAILBOX_PASSWORD")]
    pub mailbox_password: Option<String>,

    /// Pre-solved human-verification (CAPTCHA) token, skips the interactive flow.
    #[arg(long, global = true, env = "PROTON_HV_TOKEN")]
    pub captcha_token: Option<String>,

    /// Present as an official client (sets app-version + User-Agent unless overridden).
    #[arg(long, global = true, value_enum)]
    pub client: Option<ClientPreset>,

    /// Override the User-Agent header (login only).
    #[arg(long, global = true)]
    pub user_agent: Option<String>,

    /// Open the CAPTCHA in an isolated Chrome window (throwaway profile) instead
    /// of your default browser; it is closed automatically once verified.
    #[arg(long, global = true)]
    pub captcha_chrome: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// Official-client presets for `--client`.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ClientPreset {
    Web,
    Ios,
    Android,
}

impl ClientPreset {
    /// A representative `x-pm-appversion` value (override with `--app-version`).
    pub fn app_version(self) -> &'static str {
        match self {
            ClientPreset::Web => "web-mail@5.0.99.0",
            ClientPreset::Ios => "ios-mail@5.4.0",
            ClientPreset::Android => "android-mail@4.5.0",
        }
    }
    /// A representative `User-Agent` (override with `--user-agent`).
    pub fn user_agent(self) -> &'static str {
        match self {
            ClientPreset::Web => {
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
            }
            ClientPreset::Ios => "ProtonMail/5.4.0 (iOS/17.4; iPhone)",
            ClientPreset::Android => "ProtonMail/4.5.0 (Android/14; OnePlus)",
        }
    }
}

/// Global options threaded through to command handlers.
pub struct Ctx {
    pub profile: String,
    pub json: bool,
    pub api_url: Option<String>,
    pub app_version: Option<String>,
    pub totp: Option<String>,
    pub mailbox_password: Option<String>,
    pub captcha_token: Option<String>,
    pub client: Option<ClientPreset>,
    pub user_agent: Option<String>,
    pub captcha_chrome: bool,
}

impl From<&Cli> for Ctx {
    fn from(c: &Cli) -> Self {
        Ctx {
            profile: c.profile.clone(),
            json: c.json,
            api_url: c.api_url.clone(),
            app_version: c.app_version.clone(),
            totp: c.totp.clone(),
            mailbox_password: c.mailbox_password.clone(),
            captcha_token: c.captcha_token.clone(),
            client: c.client,
            user_agent: c.user_agent.clone(),
            captcha_chrome: c.captcha_chrome,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Log in to Proton Mail (prompts for credentials unless given via env).
    Login,
    /// Revoke the server session and clear local state.
    Logout,
    /// Print the primary email address of the active session.
    Whoami,
    /// Message operations.
    Messages {
        #[command(subcommand)]
        cmd: MessagesCmd,
    },
    /// Conversation (thread) operations.
    Conversations {
        #[command(subcommand)]
        cmd: ConversationsCmd,
    },
    /// Attachment operations.
    Attachments {
        #[command(subcommand)]
        cmd: AttachmentsCmd,
    },
    /// Draft operations (create/edit/delete without sending).
    Drafts {
        #[command(subcommand)]
        cmd: DraftsCmd,
    },
    /// Server-side Sieve filter operations.
    Filters {
        #[command(subcommand)]
        cmd: FiltersCmd,
    },
    /// Contacts (read).
    Contacts {
        #[command(subcommand)]
        cmd: ContactsCmd,
    },
    /// Address operations.
    Addresses {
        #[command(subcommand)]
        cmd: AddressesCmd,
    },
    /// Mail settings (read + a few toggles).
    Settings {
        #[command(subcommand)]
        cmd: SettingsCmd,
    },
    /// Per-folder message (or conversation) counts.
    Counts {
        /// Count conversations instead of messages.
        #[arg(long)]
        conversations: bool,
    },
    /// Export messages from a folder to `.eml` files.
    Export {
        #[arg(long, default_value = "all")]
        folder: String,
        /// Output directory.
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 100)]
        max: u32,
    },
    /// Continuously sync the cache (event stream) until interrupted.
    Watch {
        #[arg(long, default_value_t = 30)]
        interval: u64,
        /// Also backfill + index this folder each tick.
        #[arg(long)]
        folder: Option<String>,
    },
    /// Build the local encrypted-search index (decrypts + indexes message bodies).
    Index {
        #[arg(long, default_value = "all")]
        folder: String,
        #[arg(long = "max-pages", default_value_t = 3)]
        max_pages: u32,
        #[arg(long = "page-size", default_value_t = 50)]
        page_size: u32,
    },
    /// Full-text search the local index (message bodies; private + offline).
    Search {
        /// Query words (AND-ed together).
        query: String,
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
    /// Sync the local cache from the event stream (optionally backfill a folder).
    Sync {
        /// Also backfill this folder into the cache (e.g. `inbox`).
        #[arg(long)]
        backfill: Option<String>,
        #[arg(long = "max-pages", default_value_t = 4)]
        max_pages: u32,
        #[arg(long = "page-size", default_value_t = 50)]
        page_size: u32,
    },
    /// Label and folder operations.
    Labels {
        #[command(subcommand)]
        cmd: LabelsCmd,
    },
}

/// Shared search options (messages + conversations).
#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Free-text keyword.
    #[arg(long)]
    pub keyword: Option<String>,
    /// Match the sender address.
    #[arg(long)]
    pub from: Option<String>,
    /// Match a recipient address.
    #[arg(long)]
    pub to: Option<String>,
    /// Match the subject.
    #[arg(long)]
    pub subject: Option<String>,
    /// Only results on/after this date (YYYY-MM-DD).
    #[arg(long)]
    pub after: Option<String>,
    /// Only results on/before this date (YYYY-MM-DD).
    #[arg(long)]
    pub before: Option<String>,
    /// Restrict to a folder/label.
    #[arg(long)]
    pub folder: Option<String>,
    /// Only unread results.
    #[arg(long)]
    pub unread: bool,
    /// Maximum number of results.
    #[arg(long)]
    pub limit: Option<u32>,
}

/// Compose options shared by `send`.
#[derive(Debug, Args)]
pub struct SendArgs {
    /// Primary recipient(s) (repeatable).
    #[arg(long = "to", required = true)]
    pub to: Vec<String>,
    /// Carbon-copy recipient(s) (repeatable).
    #[arg(long = "cc")]
    pub cc: Vec<String>,
    /// Blind carbon-copy recipient(s) (repeatable).
    #[arg(long = "bcc")]
    pub bcc: Vec<String>,
    /// Sender address (defaults to the primary address).
    #[arg(long)]
    pub from: Option<String>,
    /// Message subject.
    #[arg(long)]
    pub subject: String,
    /// Message body, or `-` to read from stdin.
    #[arg(long)]
    pub body: String,
    /// Treat the body as HTML.
    #[arg(long)]
    pub html: bool,
    /// Attach a file (repeatable).
    #[arg(long = "attach")]
    pub attach: Vec<PathBuf>,
    /// Schedule delivery at this Unix timestamp.
    #[arg(long = "send-at")]
    pub send_at: Option<i64>,
    /// Self-destruct after this many seconds.
    #[arg(long = "expires")]
    pub expires: Option<i64>,
    /// Encrypted-outside: password-protect the message for keyless external recipients.
    #[arg(long = "eo-password")]
    pub eo_password: Option<String>,
    /// Password hint shown to encrypted-outside recipients.
    #[arg(long = "eo-hint")]
    pub eo_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReadFormat {
    /// Plain text with a header block.
    Text,
    /// HTML body with a header block.
    Html,
    /// Raw decrypted body only.
    Raw,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReadState {
    Read,
    Unread,
}

impl ReadState {
    pub fn as_bool(self) -> bool {
        matches!(self, ReadState::Read)
    }
}

/// Apply or remove a label.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LabelAction {
    Add,
    Rm,
}

#[derive(Debug, Subcommand)]
pub enum MessagesCmd {
    /// List messages in a folder.
    List {
        #[arg(long, default_value = "inbox")]
        folder: String,
        #[arg(long, default_value_t = 0)]
        page: u32,
        #[arg(long = "page-size", default_value_t = 25)]
        page_size: u32,
        #[arg(long)]
        unread: bool,
        /// Read from the local cache instead of the API (offline).
        #[arg(long)]
        cached: bool,
    },
    /// Search messages.
    Search(SearchArgs),
    /// Read a single message.
    Read {
        /// Message reference (ID or free-text).
        reference: String,
        #[arg(long, value_enum, default_value_t = ReadFormat::Text)]
        format: ReadFormat,
        /// Print only the body.
        #[arg(long = "body-only")]
        body_only: bool,
        /// Write the body to a file instead of stdout.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Compose and send a message.
    Send(SendArgs),
    /// Reply to a message.
    Reply {
        reference: String,
        /// Reply to all recipients.
        #[arg(long)]
        all: bool,
        #[arg(long)]
        from: Option<String>,
        /// Body text, or `-` for stdin.
        #[arg(long)]
        body: String,
        #[arg(long = "attach")]
        attach: Vec<PathBuf>,
    },
    /// Forward a message.
    Forward {
        reference: String,
        #[arg(long = "to", required = true)]
        to: Vec<String>,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        body: String,
        #[arg(long = "attach")]
        attach: Vec<PathBuf>,
    },
    /// Cancel a scheduled send.
    CancelSend { reference: String },
    /// Report messages as spam (move to Spam).
    Spam { references: Vec<String> },
    /// Report messages as not-spam (move to Inbox).
    Ham { references: Vec<String> },
    /// One-click unsubscribe from a mailing list.
    Unsubscribe { reference: String },
    /// Permanently empty a folder (e.g. `trash`, `spam`).
    Empty { folder: String },
    /// Apply or remove a label on messages.
    Label {
        #[arg(value_enum)]
        action: LabelAction,
        /// Label ID.
        label_id: String,
        /// Message references.
        references: Vec<String>,
    },
    /// Restore permanently-deleted messages (by ID).
    Undelete { ids: Vec<String> },
    /// Send a read receipt for a message.
    Receipt { reference: String },
    /// Move messages to a folder.
    Move {
        #[arg(required = true)]
        references: Vec<String>,
        #[arg(long)]
        dest: String,
    },
    /// Move messages to Trash.
    Trash {
        #[arg(required = true)]
        references: Vec<String>,
    },
    /// Permanently delete messages.
    Delete {
        #[arg(required = true)]
        references: Vec<String>,
    },
    /// Mark messages read or unread.
    Mark {
        state: ReadState,
        #[arg(required = true)]
        references: Vec<String>,
    },
    /// Star messages.
    Star {
        #[arg(required = true)]
        references: Vec<String>,
    },
    /// Unstar messages.
    Unstar {
        #[arg(required = true)]
        references: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConversationsCmd {
    /// List conversations in a folder.
    List {
        #[arg(long, default_value = "inbox")]
        folder: String,
        #[arg(long, default_value_t = 0)]
        page: u32,
        #[arg(long = "page-size", default_value_t = 25)]
        page_size: u32,
        #[arg(long)]
        unread: bool,
    },
    /// Search conversations.
    Search(SearchArgs),
    /// Read a conversation (all messages).
    Read {
        /// Conversation ID.
        id: String,
    },
    /// Move conversations to a folder.
    Move {
        #[arg(required = true)]
        ids: Vec<String>,
        #[arg(long)]
        dest: String,
    },
    /// Move conversations to Trash.
    Trash {
        #[arg(required = true)]
        ids: Vec<String>,
    },
    /// Mark conversations read or unread.
    Mark {
        state: ReadState,
        /// Label context for the mark operation.
        #[arg(long, default_value = "all")]
        folder: String,
        #[arg(required = true)]
        ids: Vec<String>,
    },
    /// Star conversations.
    Star {
        #[arg(required = true)]
        ids: Vec<String>,
    },
    /// Unstar conversations.
    Unstar {
        #[arg(required = true)]
        ids: Vec<String>,
    },
    /// Snooze conversations until a Unix timestamp (--until) or a duration (--in 1h/2d).
    Snooze {
        #[arg(required = true)]
        ids: Vec<String>,
        #[arg(long)]
        until: Option<i64>,
        /// Relative duration from now, e.g. `30m`, `2h`, `1d`.
        #[arg(long = "in")]
        relative: Option<String>,
    },
    /// Unsnooze conversations.
    Unsnooze {
        #[arg(required = true)]
        ids: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum AttachmentsCmd {
    /// List a message's attachments.
    List {
        /// Message reference (ID or free-text).
        message: String,
        #[arg(long = "include-inline")]
        include_inline: bool,
    },
    /// Download attachment(s) from a message.
    Download {
        /// Message reference (ID or free-text).
        message: String,
        /// Specific attachment ID (omit with --all).
        attachment: Option<String>,
        #[arg(long = "output-dir")]
        output_dir: Option<PathBuf>,
        /// Download all attachments.
        #[arg(long)]
        all: bool,
        #[arg(long = "include-inline")]
        include_inline: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum DraftsCmd {
    /// List drafts.
    List {
        #[arg(long, default_value_t = 0)]
        page: u32,
        #[arg(long = "page-size", default_value_t = 25)]
        page_size: u32,
    },
    /// Save a new draft (does not send).
    Save(SendArgs),
    /// Replace an existing draft's content.
    Edit {
        /// Draft message ID.
        id: String,
        #[command(flatten)]
        args: SendArgs,
    },
    /// Delete drafts.
    Delete {
        /// Draft message IDs.
        ids: Vec<String>,
    },
}

/// On/off toggle.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OnOff {
    On,
    Off,
}

impl OnOff {
    pub fn as_bool(self) -> bool {
        matches!(self, OnOff::On)
    }
}

#[derive(Debug, Subcommand)]
pub enum SettingsCmd {
    /// Show mail settings (raw JSON).
    Get,
    /// Toggle signing outgoing mail.
    Sign {
        #[arg(value_enum)]
        value: OnOff,
    },
    /// Toggle attaching your public key to outgoing mail.
    AttachPublicKey {
        #[arg(value_enum)]
        value: OnOff,
    },
}

#[derive(Debug, Subcommand)]
pub enum AddressesCmd {
    /// List addresses.
    List,
    /// Update an address (display name / signature).
    Update {
        id: String,
        #[arg(long = "display-name")]
        display_name: Option<String>,
        #[arg(long)]
        signature: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ContactsCmd {
    /// List contacts.
    List {
        #[arg(long, default_value_t = 0)]
        page: u32,
        #[arg(long = "page-size", default_value_t = 50)]
        page_size: u32,
    },
    /// List contact email addresses (optionally filter by an email).
    Emails {
        #[arg(long)]
        email: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum FiltersCmd {
    /// List filters.
    List,
    /// Create a filter from a Sieve script (`--sieve -` reads stdin).
    Create {
        #[arg(long)]
        name: String,
        /// Sieve script text, or `-` for stdin.
        #[arg(long)]
        sieve: String,
    },
    /// Validate a Sieve script without creating a filter.
    Check {
        /// Sieve script text, or `-` for stdin.
        #[arg(long)]
        sieve: String,
    },
    /// Delete a filter.
    Delete { id: String },
    /// Enable a filter.
    Enable { id: String },
    /// Disable a filter.
    Disable { id: String },
}

#[derive(Debug, Subcommand)]
pub enum LabelsCmd {
    /// List labels (or folders with --folders).
    List {
        /// List folders instead of labels.
        #[arg(long)]
        folders: bool,
    },
    /// Create a label or folder.
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "#8080FF")]
        color: String,
        /// Create a folder instead of a label.
        #[arg(long)]
        folder: bool,
        #[arg(long)]
        parent: Option<String>,
    },
    /// Delete labels/folders by ID.
    Delete {
        #[arg(required = true)]
        ids: Vec<String>,
    },
    /// Rename / recolor / reparent a label or folder.
    Update {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        color: Option<String>,
        #[arg(long)]
        parent: Option<String>,
    },
}
