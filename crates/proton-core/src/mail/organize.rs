//! Organizing: move / label / trash / delete / mark / star.

use super::Client;
use crate::api;
use crate::error::Result;
use crate::model::enums::{label_ids, label_type, resolve_folder};
use crate::model::Label;

impl Client {
    /// Move messages to a folder/label (accepts a folder name or label ID).
    pub async fn move_messages(&self, ids: &[String], folder: &str) -> Result<()> {
        api::messages::label(self.http(), &resolve_folder(folder), ids).await
    }
    /// Apply a label to messages.
    pub async fn apply_label(&self, ids: &[String], label_id: &str) -> Result<()> {
        api::messages::label(self.http(), label_id, ids).await
    }
    /// Remove a label from messages.
    pub async fn remove_label(&self, ids: &[String], label_id: &str) -> Result<()> {
        api::messages::unlabel(self.http(), label_id, ids).await
    }
    /// Move messages to Trash.
    pub async fn trash_messages(&self, ids: &[String]) -> Result<()> {
        api::messages::label(self.http(), label_ids::TRASH, ids).await
    }
    /// Permanently delete messages.
    pub async fn delete_messages(&self, ids: &[String]) -> Result<()> {
        api::messages::delete(self.http(), ids).await
    }
    /// Mark messages read or unread.
    pub async fn mark_messages_read(&self, ids: &[String], read: bool) -> Result<()> {
        if read {
            api::messages::mark_read(self.http(), ids).await
        } else {
            api::messages::mark_unread(self.http(), ids).await
        }
    }
    /// Star or unstar messages.
    pub async fn star_messages(&self, ids: &[String], starred: bool) -> Result<()> {
        if starred {
            api::messages::label(self.http(), label_ids::STARRED, ids).await
        } else {
            api::messages::unlabel(self.http(), label_ids::STARRED, ids).await
        }
    }

    // --- conversations ---
    /// Move conversations to a folder/label (accepts a folder name or label ID).
    pub async fn move_conversations(&self, ids: &[String], folder: &str) -> Result<()> {
        api::conversations::label(self.http(), &resolve_folder(folder), ids).await
    }
    /// Move conversations to Trash.
    pub async fn trash_conversations(&self, ids: &[String]) -> Result<()> {
        api::conversations::label(self.http(), label_ids::TRASH, ids).await
    }
    /// Permanently delete conversations from the given folder.
    pub async fn delete_conversations(&self, ids: &[String], folder: &str) -> Result<()> {
        api::conversations::delete(self.http(), ids, &resolve_folder(folder)).await
    }
    /// Mark conversations read or unread (unread is scoped to `folder`).
    pub async fn mark_conversations_read(
        &self,
        ids: &[String],
        read: bool,
        folder: &str,
    ) -> Result<()> {
        api::conversations::mark(self.http(), read, ids, &resolve_folder(folder)).await
    }
    /// Star or unstar conversations.
    pub async fn star_conversations(&self, ids: &[String], starred: bool) -> Result<()> {
        if starred {
            api::conversations::label(self.http(), label_ids::STARRED, ids).await
        } else {
            api::conversations::unlabel(self.http(), label_ids::STARRED, ids).await
        }
    }

    // --- labels / folders CRUD ---
    /// List the account's labels.
    pub async fn list_labels(&self) -> Result<Vec<Label>> {
        api::labels::list(self.http(), label_type::MESSAGE_LABEL).await
    }
    /// List the account's folders.
    pub async fn list_folders(&self) -> Result<Vec<Label>> {
        api::labels::list(self.http(), label_type::MESSAGE_FOLDER).await
    }
    /// Create a label, or a folder when `folder` is set, optionally nested under `parent`.
    pub async fn create_label(
        &self,
        name: &str,
        color: &str,
        folder: bool,
        parent: Option<&str>,
    ) -> Result<Label> {
        let t = if folder {
            label_type::MESSAGE_FOLDER
        } else {
            label_type::MESSAGE_LABEL
        };
        api::labels::create(self.http(), name, color, t, parent).await
    }
    /// Delete labels or folders by ID.
    pub async fn delete_labels(&self, ids: &[String]) -> Result<()> {
        api::labels::delete(self.http(), ids).await
    }

    // --- mailbox actions ---
    /// Report messages as spam (move to the Spam folder).
    pub async fn report_spam(&self, ids: &[String]) -> Result<()> {
        api::messages::label(self.http(), label_ids::SPAM, ids).await
    }
    /// Report messages as not-spam (trains the filter; server moves to Inbox).
    pub async fn report_ham(&self, ids: &[String]) -> Result<()> {
        for id in ids {
            api::messages::mark_ham(self.http(), id).await?;
        }
        Ok(())
    }
    /// One-click unsubscribe from a mailing list.
    pub async fn unsubscribe(&self, id: &str) -> Result<()> {
        api::messages::unsubscribe(self.http(), id).await
    }
    /// Permanently empty a folder (e.g. `trash`, `spam`).
    pub async fn empty_folder(&self, folder: &str) -> Result<()> {
        api::messages::empty(self.http(), &resolve_folder(folder)).await
    }
    /// Snooze conversations until a Unix timestamp.
    pub async fn snooze_conversations(&self, ids: &[String], until: i64) -> Result<()> {
        api::conversations::snooze(self.http(), ids, until).await
    }
    /// Unsnooze conversations.
    pub async fn unsnooze_conversations(&self, ids: &[String]) -> Result<()> {
        api::conversations::unsnooze(self.http(), ids).await
    }

    /// Fetch the account's mail settings (raw).
    pub async fn mail_settings(&self) -> Result<serde_json::Value> {
        api::settings::get_mail_settings(self.http()).await
    }

    /// Per-label message counts (total + unread).
    pub async fn message_counts(&self) -> Result<Vec<api::events::LabelCount>> {
        api::messages::counts(self.http()).await
    }
    /// Per-label conversation counts.
    pub async fn conversation_counts(&self) -> Result<Vec<api::events::LabelCount>> {
        api::conversations::counts(self.http()).await
    }
    /// Restore permanently-deleted messages.
    pub async fn undelete_messages(&self, ids: &[String]) -> Result<()> {
        api::messages::undelete(self.http(), ids).await
    }
    /// Rename / recolor / reparent a label or folder.
    pub async fn update_label(
        &self,
        id: &str,
        name: Option<&str>,
        color: Option<&str>,
        parent: Option<&str>,
    ) -> Result<()> {
        let mut body = serde_json::Map::new();
        if let Some(n) = name {
            body.insert("Name".into(), serde_json::json!(n));
        }
        if let Some(c) = color {
            body.insert("Color".into(), serde_json::json!(c));
        }
        if let Some(p) = parent {
            body.insert("ParentID".into(), serde_json::json!(p));
        }
        api::labels::update(self.http(), id, serde_json::Value::Object(body)).await
    }

    /// Send a read receipt for a message.
    pub async fn send_read_receipt(&self, id: &str) -> Result<()> {
        api::messages::send_receipt(self.http(), id).await
    }
    /// Toggle signing outgoing mail.
    pub async fn set_sign(&self, on: bool) -> Result<()> {
        api::settings::set_sign(self.http(), on).await
    }
    /// Toggle attaching the public key to outgoing mail.
    pub async fn set_attach_public_key(&self, on: bool) -> Result<()> {
        api::settings::set_attach_public_key(self.http(), on).await
    }
    /// Update an address (display name / signature).
    pub async fn update_address(
        &self,
        id: &str,
        display_name: Option<&str>,
        signature: Option<&str>,
    ) -> Result<()> {
        api::keys::update_address(self.http(), id, display_name, signature).await
    }
}
