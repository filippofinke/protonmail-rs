//! `create_filter` — create a Sieve mail filter (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFilterParams {
    /// Display name for the filter.
    pub name: String,
    /// The Sieve script defining the filter.
    pub sieve: String,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = create_filter_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "create_filter",
        description = "Create a mail filter from a Sieve script. Changes how incoming mail is routed: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn create_filter(
        &self,
        Parameters(p): Parameters<CreateFilterParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "create_filter",
                json!({ "name": p.name, "sieve_excerpt": excerpt(&p.sieve, 280) }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        // `proton_core::Filter` derives only `Deserialize`; build the JSON from
        // its public fields rather than serializing it directly.
        let f = client
            .create_filter(&p.name, &p.sieve)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "created": true,
            "id": f.id,
            "name": f.name,
            "status": f.status,
        })))
    }
}
