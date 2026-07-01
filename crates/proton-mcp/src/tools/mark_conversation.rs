//! `mark_conversation` — mark conversations read or unread.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MarkConversationParams {
    /// Conversation ids to update.
    pub ids: Vec<String>,
    /// Mark as read (true) or unread (false).
    pub read: bool,
    /// Label context for the mark operation. Defaults to all mail.
    pub folder: Option<String>,
}

#[tool_router(router = mark_conversation_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "mark_conversation",
        description = "Mark whole conversations (threads) read or unread."
    )]
    pub async fn mark_conversation(
        &self,
        Parameters(p): Parameters<MarkConversationParams>,
    ) -> Result<Out, ErrorData> {
        let folder = p.folder.unwrap_or_else(|| "all".to_string());

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .mark_conversations_read(&p.ids, p.read, &folder)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "updated": p.ids.len(), "ids": p.ids, "read": p.read }),
        ))
    }
}
