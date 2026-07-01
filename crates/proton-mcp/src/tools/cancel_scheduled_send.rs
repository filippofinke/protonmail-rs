//! `cancel_scheduled_send` — cancel a scheduled (delayed) send (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CancelScheduledSendParams {
    /// Message id or free text identifying the scheduled message.
    pub reference: String,
    /// Confirm the cancellation. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = cancel_scheduled_send_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "cancel_scheduled_send",
        description = "Cancel a scheduled (delayed) send. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn cancel_scheduled_send(
        &self,
        Parameters(p): Parameters<CancelScheduledSendParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "cancel_scheduled_send",
                json!({ "reference": p.reference }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let reference = self.resolve(client, &p.reference).await?;
        client
            .cancel_send(&reference)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "cancelled": true, "message_id": reference })))
    }
}
