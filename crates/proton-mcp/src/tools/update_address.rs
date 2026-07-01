//! `update_address` — update an address's display name / signature (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAddressParams {
    /// The address id to update (from list_addresses).
    pub id: String,
    /// New display name shown to recipients.
    pub display_name: Option<String>,
    /// New signature appended to outgoing mail.
    pub signature: Option<String>,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = update_address_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "update_address",
        description = "Update an address's display name and/or signature. Changes account settings: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn update_address(
        &self,
        Parameters(p): Parameters<UpdateAddressParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "update_address",
                json!({ "id": p.id, "display_name": p.display_name, "signature": p.signature }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .update_address(&p.id, p.display_name.as_deref(), p.signature.as_deref())
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "updated": true, "id": p.id })))
    }
}
