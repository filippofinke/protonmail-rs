//! `delete_messages` — permanently delete messages (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteMessagesParams {
    /// Message ids or free-text references to permanently delete.
    pub references: Vec<String>,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = delete_messages_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "delete_messages",
        description = "Permanently delete messages, bypassing Trash. IRREVERSIBLE. Prefer trash_message unless permanent deletion is intended. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn delete_messages(
        &self,
        Parameters(p): Parameters<DeleteMessagesParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "delete_messages",
                json!({
                    "references": p.references,
                    "warning": "Permanent deletion; cannot be undone.",
                }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let ids = self.resolve_all(client, &p.references).await?;
        client
            .delete_messages(&ids)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "deleted": ids.len(), "ids": ids })))
    }
}
