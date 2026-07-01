//! `unsnooze_conversation` — unsnooze conversations (return them to Inbox now).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnsnoozeConversationParams {
    /// Conversation ids to unsnooze.
    pub ids: Vec<String>,
}

#[tool_router(router = unsnooze_conversation_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "unsnooze_conversation",
        description = "Unsnooze conversations, returning them to the Inbox immediately."
    )]
    pub async fn unsnooze_conversation(
        &self,
        Parameters(p): Parameters<UnsnoozeConversationParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .unsnooze_conversations(&p.ids)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "unsnoozed": p.ids.len(), "ids": p.ids })))
    }
}
