//! Messages API.

use crate::error::Result;
use crate::model::message::{Message, MessageMetadata};
use crate::transport::{Doer, Request};
use serde::Deserialize;

/// Query parameters for listing/searching messages.
#[derive(Debug, Default, Clone)]
pub struct ListQuery {
    /// Restrict to messages with this label/folder ID.
    pub label_id: Option<String>,
    /// 0-based page number.
    pub page: Option<u32>,
    /// Number of messages per page.
    pub page_size: Option<u32>,
    /// Only return unread messages.
    pub unread: bool,
    /// Full-text keyword to search for.
    pub keyword: Option<String>,
    /// Filter by sender address.
    pub from: Option<String>,
    /// Filter by recipient address.
    pub to: Option<String>,
    /// Filter by subject.
    pub subject: Option<String>,
    /// Only include messages on or after this Unix timestamp.
    pub begin: Option<i64>,
    /// Only include messages on or before this Unix timestamp.
    pub end: Option<i64>,
    /// Restrict to messages on this address ID.
    pub address_id: Option<String>,
}

impl ListQuery {
    /// Apply to a request. `recipients=true` renames `To` → `Recipients`
    /// (used by the conversations endpoint).
    pub fn apply(&self, mut req: Request, recipients: bool) -> Request {
        req = req.query("Sort", "Time").query("Desc", "1");
        if let Some(l) = &self.label_id {
            req = req.query("LabelID", l.clone());
        }
        if let Some(p) = self.page {
            req = req.query("Page", p.to_string());
        }
        if let Some(ps) = self.page_size {
            req = req.query("PageSize", ps.to_string());
        }
        if self.unread {
            req = req.query("Unread", "1");
        }
        if let Some(k) = &self.keyword {
            req = req.query("Keyword", k.clone());
        }
        if let Some(f) = &self.from {
            req = req.query("From", f.clone());
        }
        if let Some(t) = &self.to {
            req = req.query(if recipients { "Recipients" } else { "To" }, t.clone());
        }
        if let Some(s) = &self.subject {
            req = req.query("Subject", s.clone());
        }
        if let Some(b) = self.begin {
            req = req.query("Begin", b.to_string());
        }
        if let Some(e) = self.end {
            req = req.query("End", e.to_string());
        }
        if let Some(a) = &self.address_id {
            req = req.query("AddressID", a.clone());
        }
        req
    }
}

#[derive(Deserialize)]
struct ListResp {
    #[serde(rename = "Total", default)]
    total: u32,
    #[serde(rename = "Messages", default)]
    messages: Vec<MessageMetadata>,
}

#[derive(Deserialize)]
struct GetResp {
    #[serde(rename = "Message")]
    message: Message,
}

#[derive(Deserialize)]
struct DraftResp {
    #[serde(rename = "Message")]
    message: Message,
}

/// List/search messages, returning the total count and the matching metadata.
pub async fn list_messages<D: Doer>(d: &D, q: &ListQuery) -> Result<(u32, Vec<MessageMetadata>)> {
    let req = q.apply(Request::get("/mail/v4/messages"), false);
    let r: ListResp = d.decode(req).await?;
    Ok((r.total, r.messages))
}

/// Fetch a single full message (body + attachments) by ID.
pub async fn get_message<D: Doer>(d: &D, id: &str) -> Result<Message> {
    let r: GetResp = d
        .decode(Request::get(format!("/mail/v4/messages/{id}")))
        .await?;
    Ok(r.message)
}

/// Create a draft. `body` is the full `{Message, ParentID?, Action?, AttachmentKeyPackets?}`.
pub async fn create_draft<D: Doer>(d: &D, body: serde_json::Value) -> Result<Message> {
    let r: DraftResp = d
        .decode(Request::post("/mail/v4/messages").json(body))
        .await?;
    Ok(r.message)
}

/// Update an existing draft. `body` is `{Message, AttachmentKeyPackets?}`.
pub async fn update_draft<D: Doer>(d: &D, id: &str, body: serde_json::Value) -> Result<Message> {
    let r: DraftResp = d
        .decode(Request::put(format!("/mail/v4/messages/{id}")).json(body))
        .await?;
    Ok(r.message)
}

/// Send a draft. `body` is `{ExpirationTime, AutoSaveContacts, Packages, ...}`.
pub async fn send_message<D: Doer>(d: &D, id: &str, body: serde_json::Value) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::post(format!("/mail/v4/messages/{id}")).json(body))
        .await?;
    Ok(())
}

async fn label_op<D: Doer>(d: &D, op: &str, label_id: &str, ids: &[String]) -> Result<()> {
    let body = serde_json::json!({ "LabelID": label_id, "IDs": ids });
    let _: serde_json::Value = d
        .decode(Request::put(format!("/mail/v4/messages/{op}")).json(body))
        .await?;
    Ok(())
}

/// Add a label (or move to a folder) for the given messages.
pub async fn label<D: Doer>(d: &D, label_id: &str, ids: &[String]) -> Result<()> {
    label_op(d, "label", label_id, ids).await
}
/// Remove a label from the given messages.
pub async fn unlabel<D: Doer>(d: &D, label_id: &str, ids: &[String]) -> Result<()> {
    label_op(d, "unlabel", label_id, ids).await
}

async fn ids_op<D: Doer>(d: &D, op: &str, ids: &[String]) -> Result<()> {
    let body = serde_json::json!({ "IDs": ids });
    let _: serde_json::Value = d
        .decode(Request::put(format!("/mail/v4/messages/{op}")).json(body))
        .await?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct CountsResp {
    #[serde(rename = "Counts", default)]
    counts: Vec<super::events::LabelCount>,
}

/// Per-label message counts (total + unread).
pub async fn counts<D: Doer>(d: &D) -> Result<Vec<super::events::LabelCount>> {
    let r: CountsResp = d.decode(Request::get("/mail/v4/messages/count")).await?;
    Ok(r.counts)
}

/// Undelete (restore) permanently-deleted messages.
pub async fn undelete<D: Doer>(d: &D, ids: &[String]) -> Result<()> {
    ids_op(d, "undelete", ids).await
}

/// Send a read receipt for a message.
pub async fn send_receipt<D: Doer>(d: &D, id: &str) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::post(format!("/mail/v4/messages/{id}/receipt")))
        .await?;
    Ok(())
}

/// Mark the given messages as read.
pub async fn mark_read<D: Doer>(d: &D, ids: &[String]) -> Result<()> {
    ids_op(d, "read", ids).await
}
/// Mark the given messages as unread.
pub async fn mark_unread<D: Doer>(d: &D, ids: &[String]) -> Result<()> {
    ids_op(d, "unread", ids).await
}
/// Permanently delete the given messages.
pub async fn delete<D: Doer>(d: &D, ids: &[String]) -> Result<()> {
    ids_op(d, "delete", ids).await
}

/// Cancel a scheduled send for a message.
pub async fn cancel_send<D: Doer>(d: &D, id: &str) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::post(format!("/mail/v4/messages/{id}/cancel_send")))
        .await?;
    Ok(())
}

/// Report a message as not-spam (trains the filter; server moves it to Inbox).
pub async fn mark_ham<D: Doer>(d: &D, id: &str) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::put(format!("/mail/v4/messages/{id}/mark/ham")))
        .await?;
    Ok(())
}

/// One-click unsubscribe (RFC 8058) from a mailing list.
pub async fn unsubscribe<D: Doer>(d: &D, id: &str) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::post(format!("/mail/v4/messages/{id}/unsubscribe")))
        .await?;
    Ok(())
}

/// Permanently empty a folder/label (e.g. Trash or Spam).
pub async fn empty<D: Doer>(d: &D, label_id: &str) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::delete("/mail/v4/messages/empty").query("LabelID", label_id))
        .await?;
    Ok(())
}
