//! `delete_draft` — delete draft messages (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDraftParams {
    /// Draft message ids to delete.
    pub ids: Vec<String>,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = delete_draft_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "delete_draft",
        description = "Delete draft messages by id. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn delete_draft(
        &self,
        Parameters(p): Parameters<DeleteDraftParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run("delete_draft", json!({ "ids": p.ids })));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .delete_draft(&p.ids)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "deleted": p.ids.len(), "ids": p.ids })))
    }
}
