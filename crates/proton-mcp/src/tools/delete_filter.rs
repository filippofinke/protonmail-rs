//! `delete_filter` — delete a mail filter (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteFilterParams {
    /// The filter id to delete (from list_filters).
    pub id: String,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = delete_filter_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "delete_filter",
        description = "Delete a mail filter by id. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn delete_filter(
        &self,
        Parameters(p): Parameters<DeleteFilterParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run("delete_filter", json!({ "id": p.id })));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .delete_filter(&p.id)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "deleted": true, "id": p.id })))
    }
}
