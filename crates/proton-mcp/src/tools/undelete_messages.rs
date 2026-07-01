//! `undelete_messages` — restore permanently-deleted messages (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UndeleteMessagesParams {
    /// Message ids to restore. These are exact ids — deleted messages are not
    /// searchable, so free-text references cannot be resolved here.
    pub ids: Vec<String>,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = undelete_messages_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "undelete_messages",
        description = "Restore permanently-deleted messages by id. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn undelete_messages(
        &self,
        Parameters(p): Parameters<UndeleteMessagesParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run("undelete_messages", json!({ "ids": p.ids })));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .undelete_messages(&p.ids)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "undeleted": p.ids.len(), "ids": p.ids })))
    }
}
