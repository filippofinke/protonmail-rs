//! `star_conversation` — star or unstar conversations.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StarConversationParams {
    /// Conversation ids to update.
    pub ids: Vec<String>,
    /// Star (true) or unstar (false).
    pub starred: bool,
}

#[tool_router(router = star_conversation_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "star_conversation",
        description = "Star or unstar whole conversations (threads)."
    )]
    pub async fn star_conversation(
        &self,
        Parameters(p): Parameters<StarConversationParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .star_conversations(&p.ids, p.starred)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "updated": p.ids.len(), "ids": p.ids, "starred": p.starred }),
        ))
    }
}
