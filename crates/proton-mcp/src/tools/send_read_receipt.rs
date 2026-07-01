//! `send_read_receipt` — send a read receipt for a message (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SendReadReceiptParams {
    /// Message id or free-text reference that requested a read receipt.
    pub reference: String,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = send_read_receipt_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "send_read_receipt",
        description = "Send a read receipt for a message that requested one (notifies the sender you read it). Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn send_read_receipt(
        &self,
        Parameters(p): Parameters<SendReadReceiptParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "send_read_receipt",
                json!({ "reference": p.reference }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let id = self.resolve(client, &p.reference).await?;
        client
            .send_read_receipt(&id)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "receipt_sent": true, "id": id })))
    }
}
