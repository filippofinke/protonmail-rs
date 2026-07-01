//! `snooze_conversation` — snooze conversations until a timestamp.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SnoozeConversationParams {
    /// Conversation ids to snooze.
    pub ids: Vec<String>,
    /// Unix timestamp (seconds) to snooze until; the thread reappears in Inbox then.
    pub until: i64,
}

#[tool_router(router = snooze_conversation_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "snooze_conversation",
        description = "Snooze conversations until a Unix timestamp (they leave Inbox and return at that time)."
    )]
    pub async fn snooze_conversation(
        &self,
        Parameters(p): Parameters<SnoozeConversationParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .snooze_conversations(&p.ids, p.until)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "snoozed": p.ids.len(), "ids": p.ids, "until": p.until }),
        ))
    }
}
