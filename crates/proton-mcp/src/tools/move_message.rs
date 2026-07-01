//! `move_message` — move messages to a folder/label (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveMessageParams {
    /// Message ids or free-text references to move.
    pub references: Vec<String>,
    /// Destination folder/label (e.g. archive, trash, spam, inbox, or a label id).
    pub folder: String,
    /// Confirm the move. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = move_message_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "move_message",
        description = "Move messages to a folder/label. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn move_message(
        &self,
        Parameters(p): Parameters<MoveMessageParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "move_message",
                json!({ "references": p.references, "folder": p.folder }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let ids = self.resolve_all(client, &p.references).await?;
        client
            .move_messages(&ids, &p.folder)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "moved": ids.len(), "ids": ids, "folder": p.folder }),
        ))
    }
}
