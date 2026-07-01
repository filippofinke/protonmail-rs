//! `counts` — per-folder message (or conversation) counts.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CountsParams {
    /// Count conversations instead of messages. Defaults to false (messages).
    pub conversations: Option<bool>,
}

#[tool_router(router = counts_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "counts",
        description = "Per-folder/label counts (total + unread) for messages, or for conversations when conversations=true."
    )]
    pub async fn counts(&self, Parameters(p): Parameters<CountsParams>) -> Result<Out, ErrorData> {
        let by_conversation = p.conversations.unwrap_or(false);

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let counts = if by_conversation {
            client.conversation_counts().await
        } else {
            client.message_counts().await
        }
        .map_err(|e| self.map_err(e))?;

        Ok(obj(json!({
            "kind": if by_conversation { "conversations" } else { "messages" },
            "count": counts.len(),
            "counts": to_value(&counts)?,
        })))
    }
}
