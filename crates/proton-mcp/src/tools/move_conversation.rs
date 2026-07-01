//! `move_conversation` — move conversations to a folder/label (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveConversationParams {
    /// Conversation ids to move.
    pub ids: Vec<String>,
    /// Destination folder/label (e.g. archive, trash, spam, inbox, or a label id).
    pub folder: String,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = move_conversation_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "move_conversation",
        description = "Move whole conversations (threads) to a folder/label. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn move_conversation(
        &self,
        Parameters(p): Parameters<MoveConversationParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "move_conversation",
                json!({ "ids": p.ids, "folder": p.folder }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .move_conversations(&p.ids, &p.folder)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "moved": p.ids.len(), "ids": p.ids, "folder": p.folder }),
        ))
    }
}
