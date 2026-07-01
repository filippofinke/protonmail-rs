//! Conversations API.

use super::messages::ListQuery;
use crate::error::Result;
use crate::model::conversation::Conversation;
use crate::model::message::Message;
use crate::transport::{Doer, Request};
use serde::Deserialize;

#[derive(Deserialize)]
struct ListResp {
    #[serde(rename = "Total", default)]
    total: u32,
    #[serde(rename = "Conversations", default)]
    conversations: Vec<Conversation>,
}

#[derive(Deserialize)]
struct GetResp {
    #[serde(rename = "Conversation")]
    conversation: Conversation,
    #[serde(rename = "Messages", default)]
    messages: Vec<Message>,
}

/// List/search conversations, returning the total count and the matches.
pub async fn list_conversations<D: Doer>(d: &D, q: &ListQuery) -> Result<(u32, Vec<Conversation>)> {
    let req = q.apply(Request::get("/mail/v4/conversations"), true);
    let r: ListResp = d.decode(req).await?;
    Ok((r.total, r.conversations))
}

/// Fetch a conversation and its messages by ID.
pub async fn get_conversation<D: Doer>(d: &D, id: &str) -> Result<(Conversation, Vec<Message>)> {
    let r: GetResp = d
        .decode(Request::get(format!("/mail/v4/conversations/{id}")))
        .await?;
    Ok((r.conversation, r.messages))
}

async fn label_op<D: Doer>(d: &D, op: &str, label_id: &str, ids: &[String]) -> Result<()> {
    let body = serde_json::json!({ "LabelID": label_id, "IDs": ids });
    let _: serde_json::Value = d
        .decode(Request::put(format!("/mail/v4/conversations/{op}")).json(body))
        .await?;
    Ok(())
}

/// Add a label (or move to a folder) for the given conversations.
pub async fn label<D: Doer>(d: &D, label_id: &str, ids: &[String]) -> Result<()> {
    label_op(d, "label", label_id, ids).await
}
/// Remove a label from the given conversations.
pub async fn unlabel<D: Doer>(d: &D, label_id: &str, ids: &[String]) -> Result<()> {
    label_op(d, "unlabel", label_id, ids).await
}

/// Mark conversations read/unread. Unread also takes a `LabelID`.
pub async fn mark<D: Doer>(d: &D, read: bool, ids: &[String], label_id: &str) -> Result<()> {
    let (op, body) = if read {
        ("read", serde_json::json!({ "IDs": ids }))
    } else {
        (
            "unread",
            serde_json::json!({ "IDs": ids, "LabelID": label_id }),
        )
    };
    let _: serde_json::Value = d
        .decode(Request::put(format!("/mail/v4/conversations/{op}")).json(body))
        .await?;
    Ok(())
}

/// Permanently delete the given conversations from a folder.
pub async fn delete<D: Doer>(d: &D, ids: &[String], label_id: &str) -> Result<()> {
    let body = serde_json::json!({ "IDs": ids, "LabelID": label_id });
    let _: serde_json::Value = d
        .decode(Request::put("/mail/v4/conversations/delete").json(body))
        .await?;
    Ok(())
}

/// Snooze conversations until a Unix timestamp.
pub async fn snooze<D: Doer>(d: &D, ids: &[String], snooze_time: i64) -> Result<()> {
    let body = serde_json::json!({ "IDs": ids, "SnoozeTime": snooze_time });
    let _: serde_json::Value = d
        .decode(Request::put("/mail/v4/conversations/snooze").json(body))
        .await?;
    Ok(())
}

/// Unsnooze the given conversations.
pub async fn unsnooze<D: Doer>(d: &D, ids: &[String]) -> Result<()> {
    let body = serde_json::json!({ "IDs": ids });
    let _: serde_json::Value = d
        .decode(Request::put("/mail/v4/conversations/unsnooze").json(body))
        .await?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct CountsResp {
    #[serde(rename = "Counts", default)]
    counts: Vec<super::events::LabelCount>,
}

/// Per-label conversation counts.
pub async fn counts<D: Doer>(d: &D) -> Result<Vec<super::events::LabelCount>> {
    let r: CountsResp = d
        .decode(Request::get("/mail/v4/conversations/count"))
        .await?;
    Ok(r.counts)
}
