//! `forward_message` — forward a message to new recipients (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use proton_core::SendOptions;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ForwardMessageParams {
    /// Message id or free text identifying the message to forward.
    pub reference: String,
    /// Recipient addresses.
    pub to: Vec<String>,
    /// Sender address. Defaults to the primary address.
    pub from: Option<String>,
    /// Optional extra body to prepend.
    pub body: String,
    /// Treat the body as HTML. Defaults to false.
    pub html: Option<bool>,
    /// Confirm the (destructive) send. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = forward_message_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "forward_message",
        description = "Forward a message to new recipients. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn forward_message(
        &self,
        Parameters(p): Parameters<ForwardMessageParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "forward_message",
                json!({
                    "reference": p.reference,
                    "to": p.to,
                    "from": p.from,
                    "body_excerpt": excerpt(&p.body, 280),
                }),
            ));
        }

        let opts = SendOptions {
            to: p.to,
            from: p.from,
            body: p.body,
            html: p.html.unwrap_or(false),
            ..Default::default()
        };

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let reference = self.resolve(client, &p.reference).await?;
        let id = client
            .forward(&reference, &opts)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "sent": true, "message_id": id })))
    }
}
