//! `set_attach_public_key` — toggle attaching your public key to outgoing mail (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetAttachPublicKeyParams {
    /// Enable (true) or disable (false) attaching your public key to outgoing mail.
    pub enabled: bool,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = set_attach_public_key_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "set_attach_public_key",
        description = "Toggle attaching your public key to outgoing mail. Changes account settings: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn set_attach_public_key(
        &self,
        Parameters(p): Parameters<SetAttachPublicKeyParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "set_attach_public_key",
                json!({ "enabled": p.enabled }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .set_attach_public_key(p.enabled)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "attach_public_key": p.enabled })))
    }
}
