//! `enable_filter` — enable a mail filter (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnableFilterParams {
    /// The filter id to enable.
    pub id: String,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = enable_filter_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "enable_filter",
        description = "Enable a mail filter by id. Changes mail routing: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn enable_filter(
        &self,
        Parameters(p): Parameters<EnableFilterParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run("enable_filter", json!({ "id": p.id })));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .enable_filter(&p.id)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "enabled": true, "id": p.id })))
    }
}
