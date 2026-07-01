//! `unsubscribe` — one-click unsubscribe from a mailing list (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnsubscribeParams {
    /// Message id or free-text reference to a list message carrying unsubscribe info.
    pub reference: String,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = unsubscribe_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "unsubscribe",
        description = "One-click unsubscribe from the mailing list of a message (List-Unsubscribe). Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn unsubscribe(
        &self,
        Parameters(p): Parameters<UnsubscribeParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run("unsubscribe", json!({ "reference": p.reference })));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let id = self.resolve(client, &p.reference).await?;
        client.unsubscribe(&id).await.map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "unsubscribed": true, "id": id })))
    }
}
