//! `trash_conversation` — move conversations to Trash (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrashConversationParams {
    /// Conversation ids to trash.
    pub ids: Vec<String>,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = trash_conversation_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "trash_conversation",
        description = "Move whole conversations (threads) to Trash. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn trash_conversation(
        &self,
        Parameters(p): Parameters<TrashConversationParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run("trash_conversation", json!({ "ids": p.ids })));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .trash_conversations(&p.ids)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "trashed": p.ids.len(), "ids": p.ids })))
    }
}
