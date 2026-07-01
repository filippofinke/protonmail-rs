//! `reply_message` — reply to a message (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use proton_core::SendOptions;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReplyMessageParams {
    /// Message id or free text identifying the message to reply to.
    pub reference: String,
    /// Reply to all recipients. Defaults to false (sender only).
    pub all: Option<bool>,
    /// Sender address. Defaults to the primary address.
    pub from: Option<String>,
    /// Reply body.
    pub body: String,
    /// Treat the body as HTML. Defaults to false.
    pub html: Option<bool>,
    /// Confirm the (destructive) send. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = reply_message_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "reply_message",
        description = "Reply to a message. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn reply_message(
        &self,
        Parameters(p): Parameters<ReplyMessageParams>,
    ) -> Result<Out, ErrorData> {
        let all = p.all.unwrap_or(false);
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "reply_message",
                json!({
                    "reference": p.reference,
                    "all": all,
                    "from": p.from,
                    "body_excerpt": excerpt(&p.body, 280),
                }),
            ));
        }

        let opts = SendOptions {
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
            .reply(&reference, all, &opts)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "sent": true, "message_id": id })))
    }
}
