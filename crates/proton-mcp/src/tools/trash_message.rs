//! `trash_message` — move messages to Trash (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrashMessageParams {
    /// Message ids or free-text references to trash.
    pub references: Vec<String>,
    /// Confirm trashing. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = trash_message_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "trash_message",
        description = "Move messages to Trash. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn trash_message(
        &self,
        Parameters(p): Parameters<TrashMessageParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "trash_message",
                json!({ "references": p.references }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let ids = self.resolve_all(client, &p.references).await?;
        client
            .trash_messages(&ids)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "trashed": ids.len(), "ids": ids })))
    }
}
