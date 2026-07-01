//! `send_message` — compose and send a new message (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use proton_core::SendOptions;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SendMessageParams {
    /// Primary recipient addresses.
    pub to: Vec<String>,
    /// CC recipient addresses.
    #[serde(default)]
    pub cc: Vec<String>,
    /// BCC recipient addresses.
    #[serde(default)]
    pub bcc: Vec<String>,
    /// Sender address (must be one of your addresses). Defaults to the primary address.
    pub from: Option<String>,
    /// Subject line.
    pub subject: String,
    /// Message body.
    pub body: String,
    /// Treat the body as HTML. Defaults to false (plain text).
    pub html: Option<bool>,
    /// Unix timestamp (seconds) to schedule delivery.
    pub send_at: Option<i64>,
    /// Self-destruct lifetime in seconds.
    pub expires: Option<i64>,
    /// Confirm the (destructive) send. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = send_message_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "send_message",
        description = "Compose and send a new message. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn send_message(
        &self,
        Parameters(p): Parameters<SendMessageParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "send_message",
                json!({
                    "to": p.to,
                    "cc": p.cc,
                    "bcc": p.bcc,
                    "from": p.from,
                    "subject": p.subject,
                    "body_excerpt": excerpt(&p.body, 280),
                    "html": p.html.unwrap_or(false),
                    "send_at": p.send_at,
                    "expires": p.expires,
                }),
            ));
        }

        let opts = SendOptions {
            to: p.to,
            cc: p.cc,
            bcc: p.bcc,
            from: p.from,
            subject: p.subject,
            body: p.body,
            html: p.html.unwrap_or(false),
            attachments: Vec::new(),
            send_at: p.send_at,
            expires_in: p.expires,
        };

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let id = client.send(&opts).await.map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "sent": true, "message_id": id })))
    }
}
