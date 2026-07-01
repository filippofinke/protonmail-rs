//! Message + attachment models (serde, PascalCase JSON).

use serde::{Deserialize, Serialize};

/// A mail recipient / sender.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Recipient {
    /// Display name (defaults to the address).
    #[serde(default)]
    pub name: String,
    /// Email address.
    pub address: String,
    /// Contact ID, if this recipient is a saved contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<String>,
    /// `1` if the recipient is a Proton address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_proton: Option<i64>,
}

impl Recipient {
    /// Build a recipient from an address (using the address as the display name).
    pub fn new(address: impl Into<String>) -> Self {
        let address = address.into();
        Recipient {
            name: address.clone(),
            address,
            contact_id: None,
            is_proton: None,
        }
    }
}

/// Attachment metadata (and crypto material) as returned on a message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Attachment {
    /// Attachment ID.
    #[serde(default)]
    pub id: String,
    /// File name.
    #[serde(default)]
    pub name: String,
    /// Size in bytes.
    #[serde(default)]
    pub size: u64,
    /// Encrypted session-key packets (base64), used to decrypt the attachment.
    #[serde(rename = "KeyPackets", default)]
    pub key_packets: Option<String>,
    /// MIME type of the attachment.
    #[serde(rename = "MIMEType", default)]
    pub mime_type: Option<String>,
    /// Content disposition (`inline` or `attachment`).
    #[serde(default)]
    pub disposition: Option<String>,
    /// Detached signature over the attachment, if present.
    #[serde(default)]
    pub signature: Option<String>,
}

impl Attachment {
    /// True if the attachment is inline (disposition `inline`) rather than a download.
    pub fn is_inline(&self) -> bool {
        self.disposition.as_deref() == Some("inline")
    }
}

/// Message metadata (list/search results).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageMetadata {
    /// Message ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Sort order within the listing.
    #[serde(default)]
    pub order: i64,
    /// ID of the conversation this message belongs to.
    #[serde(rename = "ConversationID", default)]
    pub conversation_id: String,
    /// Subject line.
    #[serde(default)]
    pub subject: String,
    /// `1` if unread, `0` if read.
    #[serde(default)]
    pub unread: i64,
    /// Sender.
    #[serde(default)]
    pub sender: Recipient,
    /// Primary (To) recipients.
    #[serde(default)]
    pub to_list: Vec<Recipient>,
    /// Carbon-copy (CC) recipients.
    #[serde(rename = "CCList", default)]
    pub cc_list: Vec<Recipient>,
    /// Blind carbon-copy (BCC) recipients.
    #[serde(rename = "BCCList", default)]
    pub bcc_list: Vec<Recipient>,
    /// Unix timestamp the message was received.
    #[serde(default)]
    pub time: i64,
    /// Total message size in bytes.
    #[serde(default)]
    pub size: u64,
    /// ID of the address that owns this message.
    #[serde(rename = "AddressID", default)]
    pub address_id: String,
    /// IDs of the labels/folders applied to the message.
    #[serde(rename = "LabelIDs", default)]
    pub label_ids: Vec<String>,
    /// Original RFC822 `Message-ID`, if any.
    #[serde(rename = "ExternalID", default)]
    pub external_id: Option<String>,
    /// Number of attachments.
    #[serde(default)]
    pub num_attachments: i64,
    /// Bitmask — MUST be u64 (high bits overflow 32-bit).
    #[serde(default)]
    pub flags: u64,
    /// Unix timestamp at which the message expires, if set.
    #[serde(default)]
    pub expiration_time: Option<i64>,
}

/// A full message (adds body + crypto-bearing fields).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message {
    /// Shared metadata (flattened into the same JSON object).
    #[serde(flatten)]
    pub meta: MessageMetadata,
    /// Encrypted message body (armored PGP).
    #[serde(default)]
    pub body: String,
    /// MIME type of the body.
    #[serde(rename = "MIMEType", default)]
    pub mime_type: String,
    /// Raw RFC822 headers, if requested.
    #[serde(default)]
    pub header: Option<String>,
    /// Attachments.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// Reply-To address, if set.
    #[serde(default)]
    pub reply_to: Option<Recipient>,
    /// Reply-To addresses.
    #[serde(default)]
    pub reply_tos: Vec<Recipient>,
    /// Encrypted-outside password, for password-protected messages.
    #[serde(default)]
    pub password: Option<String>,
    /// Hint for the encrypted-outside password.
    #[serde(default)]
    pub password_hint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_message_with_high_flag() {
        // Flags carries 2^35 (auto-forwardee) — must not overflow.
        let json = serde_json::json!({
            "ID": "abc",
            "ConversationID": "conv1",
            "Subject": "hi",
            "Unread": 1,
            "Sender": {"Name": "A", "Address": "a@proton.me"},
            "ToList": [{"Name":"B","Address":"b@x.com"}],
            "Time": 1700000000,
            "Flags": 34359738372i64, // 2^35 + 2^2
            "MIMEType": "text/html",
            "Body": "ARMORED",
            "Attachments": [{"ID":"att1","Name":"f.pdf","KeyPackets":"KP","MIMEType":"application/pdf"}]
        });
        let m: Message = serde_json::from_value(json).unwrap();
        assert_eq!(m.meta.id, "abc");
        assert_eq!(m.meta.conversation_id, "conv1");
        assert_eq!(m.meta.flags, (1u64 << 35) | (1u64 << 2));
        assert_eq!(m.mime_type, "text/html");
        assert_eq!(m.meta.to_list.len(), 1);
        assert_eq!(m.attachments.len(), 1);
        assert_eq!(m.attachments[0].key_packets.as_deref(), Some("KP"));
    }
}
