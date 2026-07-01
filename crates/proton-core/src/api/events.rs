//! Event-stream API (`/core/v4/events`) for incremental sync.

use crate::error::Result;
use crate::model::message::MessageMetadata;
use crate::transport::{Doer, Request};
use serde::Deserialize;

/// Event action codes.
pub mod action {
    /// The item was deleted.
    pub const DELETE: u8 = 0;
    /// The item was created.
    pub const CREATE: u8 = 1;
    /// The item was updated (full object).
    pub const UPDATE: u8 = 2;
    /// Only the item's flags/labels changed.
    pub const UPDATE_FLAGS: u8 = 3;
}

#[derive(Deserialize)]
struct LatestResp {
    #[serde(rename = "EventID")]
    event_id: String,
}

/// A single message change in an event batch.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageEvent {
    /// Message ID this change applies to.
    #[serde(rename = "ID")]
    pub id: String,
    /// Action code (see the `action` module).
    #[serde(rename = "Action")]
    pub action: u8,
    /// Updated message metadata, when the action carries it.
    #[serde(rename = "Message", default)]
    pub message: Option<MessageMetadata>,
}

/// A per-label count update.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct LabelCount {
    /// Label/folder ID.
    #[serde(rename = "LabelID")]
    pub label_id: String,
    /// Total number of messages with this label.
    #[serde(rename = "Total", default)]
    pub total: i64,
    /// Number of unread messages with this label.
    #[serde(rename = "Unread", default)]
    pub unread: i64,
}

#[derive(Deserialize)]
struct EventResp {
    #[serde(rename = "EventID")]
    event_id: String,
    #[serde(rename = "More", default)]
    more: i64,
    #[serde(rename = "Refresh", default)]
    refresh: i64,
    #[serde(rename = "Messages", default)]
    messages: Vec<MessageEvent>,
    #[serde(rename = "MessageCounts", default)]
    message_counts: Vec<LabelCount>,
}

/// A decoded event batch.
#[derive(Debug, Clone)]
pub struct EventBatch {
    /// Cursor to pass to the next `get_events` call.
    pub event_id: String,
    /// More events are immediately available (loop until false).
    pub more: bool,
    /// The server asked for a full resync (drop the cache and re-bootstrap).
    pub refresh: bool,
    /// Message changes in this batch.
    pub messages: Vec<MessageEvent>,
    /// Per-label count updates in this batch.
    pub counts: Vec<LabelCount>,
}

/// The current latest event ID (the sync cursor start).
pub async fn get_latest_event_id<D: Doer>(d: &D) -> Result<String> {
    let r: LatestResp = d.decode(Request::get("/core/v4/events/latest")).await?;
    Ok(r.event_id)
}

/// Fetch the batch of events since `event_id`.
pub async fn get_events<D: Doer>(d: &D, event_id: &str) -> Result<EventBatch> {
    let r: EventResp = d
        .decode(Request::get(format!("/core/v4/events/{event_id}")))
        .await?;
    Ok(EventBatch {
        event_id: r.event_id,
        more: r.more == 1,
        refresh: r.refresh & 1 != 0,
        messages: r.messages,
        counts: r.message_counts,
    })
}
