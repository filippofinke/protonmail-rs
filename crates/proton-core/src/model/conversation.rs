//! Conversation models.

use super::message::Recipient;
use serde::{Deserialize, Serialize};

/// A label applied to a conversation, with per-label context counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConversationLabel {
    /// Label ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Number of messages in the conversation under this label.
    #[serde(default)]
    pub context_num_messages: Option<i64>,
    /// Number of unread messages in the conversation under this label.
    #[serde(default)]
    pub context_num_unread: Option<i64>,
    /// Unix timestamp of the latest message under this label.
    #[serde(default)]
    pub context_time: Option<i64>,
}

/// A conversation (message thread) with its summary metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Conversation {
    /// Conversation ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Subject line.
    #[serde(default)]
    pub subject: String,
    /// Unix timestamp of the most recent message.
    #[serde(default)]
    pub time: i64,
    /// Total number of messages in the conversation.
    #[serde(default)]
    pub num_messages: i64,
    /// Number of unread messages.
    #[serde(default)]
    pub num_unread: i64,
    /// Total number of attachments across the conversation.
    #[serde(default)]
    pub num_attachments: i64,
    /// Distinct senders in the conversation.
    #[serde(default)]
    pub senders: Vec<Recipient>,
    /// Distinct recipients in the conversation.
    #[serde(default)]
    pub recipients: Vec<Recipient>,
    /// Labels applied to the conversation, with per-label counts.
    #[serde(default)]
    pub labels: Vec<ConversationLabel>,
    /// Unix timestamp at which the conversation expires, if set.
    #[serde(default)]
    pub expiration_time: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_conversation() {
        let json = serde_json::json!({
            "ID": "c1", "Subject": "thread", "NumMessages": 3, "NumUnread": 1,
            "Senders": [{"Name":"A","Address":"a@proton.me"}],
            "Labels": [{"ID":"0"}]
        });
        let c: Conversation = serde_json::from_value(json).unwrap();
        assert_eq!(c.id, "c1");
        assert_eq!(c.num_messages, 3);
        assert_eq!(c.senders.len(), 1);
        assert_eq!(c.labels[0].id, "0");
    }
}
