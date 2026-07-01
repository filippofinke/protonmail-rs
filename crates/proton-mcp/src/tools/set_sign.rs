//! `set_sign` — toggle signing of outgoing mail (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetSignParams {
    /// Enable (true) or disable (false) signing outgoing mail.
    pub enabled: bool,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = set_sign_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "set_sign",
        description = "Toggle cryptographic signing of outgoing mail. Changes account settings: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn set_sign(
        &self,
        Parameters(p): Parameters<SetSignParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run("set_sign", json!({ "enabled": p.enabled })));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .set_sign(p.enabled)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "sign": p.enabled })))
    }
}
