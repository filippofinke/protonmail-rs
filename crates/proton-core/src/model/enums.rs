//! Load-bearing Proton Mail enums and constants (verified against the spec §7.4).

/// System mailbox label IDs (string values, as the API uses them).
pub mod label_ids {
    /// Inbox.
    pub const INBOX: &str = "0";
    /// All drafts (across every folder).
    pub const ALL_DRAFTS: &str = "1";
    /// All sent messages (across every folder).
    pub const ALL_SENT: &str = "2";
    /// Trash.
    pub const TRASH: &str = "3";
    /// Spam.
    pub const SPAM: &str = "4";
    /// All mail.
    pub const ALL_MAIL: &str = "5";
    /// Archive.
    pub const ARCHIVE: &str = "6";
    /// Sent.
    pub const SENT: &str = "7";
    /// Drafts.
    pub const DRAFTS: &str = "8";
    /// Outbox.
    pub const OUTBOX: &str = "9";
    /// Starred.
    pub const STARRED: &str = "10";
    /// Scheduled send.
    pub const SCHEDULED: &str = "12";
    /// Almost all mail (everything except Spam and Trash).
    pub const ALMOST_ALL_MAIL: &str = "15";
    /// Snoozed.
    pub const SNOOZED: &str = "16";
}

/// Resolve a human folder name to its label ID. Unknown names pass through
/// unchanged (so raw label IDs work anywhere).
pub fn resolve_folder(name: &str) -> String {
    match name.trim().to_ascii_lowercase().as_str() {
        "inbox" => label_ids::INBOX,
        "drafts" => label_ids::DRAFTS,
        "sent" => label_ids::SENT,
        "trash" => label_ids::TRASH,
        "spam" => label_ids::SPAM,
        "archive" => label_ids::ARCHIVE,
        "starred" => label_ids::STARRED,
        "all" | "all-mail" | "allmail" => label_ids::ALL_MAIL,
        "scheduled" => label_ids::SCHEDULED,
        "snoozed" => label_ids::SNOOZED,
        other => return other.to_string(),
    }
    .to_string()
}

/// Send package types (bitmask).
pub mod package_type {
    /// Internal Proton (end-to-end encrypted).
    pub const PM: i64 = 1; // internal Proton
    /// Encrypted-outside (password-protected).
    pub const EO: i64 = 2; // encrypted-outside (password)
    /// Cleartext (unencrypted).
    pub const CLEAR: i64 = 4;
    /// PGP inline.
    pub const PGP_INLINE: i64 = 8;
    /// PGP/MIME.
    pub const PGP_MIME: i64 = 16;
    /// Cleartext MIME.
    pub const CLEAR_MIME: i64 = 32;
}

/// Address-key flags (bitmask).
pub mod key_flag {
    /// Key is not compromised (may verify signatures).
    pub const NOT_COMPROMISED: u32 = 1; // may verify signatures
    /// Key is not obsolete (may encrypt).
    pub const NOT_OBSOLETE: u32 = 2; // may encrypt
    /// Email encryption is disabled for this address.
    pub const EMAIL_NO_ENCRYPT: u32 = 4;
    /// Email signing is disabled for this address.
    pub const EMAIL_NO_SIGN: u32 = 8;

    /// True if the key may be used to encrypt to this recipient.
    pub fn can_encrypt(flags: u32) -> bool {
        flags & NOT_OBSOLETE != 0
    }
}

/// Message flags (bitmask; `u64` because high bits overflow 32-bit).
pub mod message_flag {
    /// Message was received.
    pub const RECEIVED: u64 = 1 << 0;
    /// Message was sent.
    pub const SENT: u64 = 1 << 1;
    /// Message is internal (Proton-to-Proton).
    pub const INTERNAL: u64 = 1 << 2;
    /// Message is end-to-end encrypted.
    pub const E2E: u64 = 1 << 3;
    /// Message has been replied to.
    pub const REPLIED: u64 = 1 << 5;
    /// Message has been replied to all.
    pub const REPLIED_ALL: u64 = 1 << 6;
    /// Message has been forwarded.
    pub const FORWARDED: u64 = 1 << 7;
    /// Message carries a public key.
    pub const PUBLIC_KEY: u64 = 1 << 17;
    /// Message is signed.
    pub const SIGN: u64 = 1 << 18;
    /// Message is a scheduled send.
    pub const SCHEDULED_SEND: u64 = 1 << 20;
    /// Message was produced by auto-forwarding.
    pub const AUTO_FORWARDEE: u64 = 1 << 35;

    /// True if `flags` has `flag` set.
    pub fn has(flags: u64, flag: u64) -> bool {
        flags & flag != 0
    }
}

/// Recipient type from the public-key lookup.
pub mod recipient_type {
    /// Internal Proton recipient.
    pub const INTERNAL: u8 = 1;
    /// External recipient.
    pub const EXTERNAL: u8 = 2;
}

/// Label types for `/core/v4/labels`.
pub mod label_type {
    /// Message label.
    pub const MESSAGE_LABEL: i64 = 1;
    /// Contact group.
    pub const CONTACT_GROUP: i64 = 2;
    /// Message folder (user-created).
    pub const MESSAGE_FOLDER: i64 = 3;
    /// System folder.
    pub const SYSTEM_FOLDER: i64 = 4;
}

/// MIME body types used by mail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MimeType {
    /// `text/plain`.
    PlainText,
    /// `text/html`.
    Html,
    /// `multipart/mixed`.
    MultipartMixed,
}

impl MimeType {
    /// The MIME type's wire string (e.g. `text/plain`).
    pub fn as_str(self) -> &'static str {
        match self {
            MimeType::PlainText => "text/plain",
            MimeType::Html => "text/html",
            MimeType::MultipartMixed => "multipart/mixed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_resolution() {
        assert_eq!(resolve_folder("inbox"), "0");
        assert_eq!(resolve_folder("INBOX"), "0");
        assert_eq!(resolve_folder("trash"), "3");
        assert_eq!(resolve_folder("sent"), "7");
        assert_eq!(resolve_folder("starred"), "10");
        assert_eq!(resolve_folder("all"), "5");
        assert_eq!(resolve_folder("xyz123"), "xyz123");
    }

    #[test]
    fn flag_values() {
        assert_eq!(key_flag::NOT_OBSOLETE, 2);
        assert!(key_flag::can_encrypt(
            key_flag::NOT_OBSOLETE | key_flag::NOT_COMPROMISED
        ));
        assert!(!key_flag::can_encrypt(key_flag::NOT_COMPROMISED));
        assert_eq!(package_type::PM, 1);
        assert_eq!(package_type::CLEAR, 4);
        assert_eq!(message_flag::AUTO_FORWARDEE, 1 << 35);
        assert!(message_flag::has(
            message_flag::AUTO_FORWARDEE,
            message_flag::AUTO_FORWARDEE
        ));
    }
}
