//! `save_draft` — compose and save a draft message (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use proton_core::SendOptions;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveDraftParams {
    /// Primary recipient addresses.
    pub to: Vec<String>,
    /// CC recipient addresses.
    pub cc: Option<Vec<String>>,
    /// BCC recipient addresses.
    pub bcc: Option<Vec<String>>,
    /// Sender address (must be one of your addresses). Defaults to the primary address.
    pub from: Option<String>,
    /// Subject line.
    pub subject: String,
    /// Message body.
    pub body: String,
    /// Treat the body as HTML. Defaults to false (plain text).
    pub html: Option<bool>,
    /// Confirm the (data-creating) save. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = save_draft_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "save_draft",
        description = "Compose and save a draft message. Creates data: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn save_draft(
        &self,
        Parameters(p): Parameters<SaveDraftParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "save_draft",
                json!({
                    "to": p.to,
                    "cc": p.cc,
                    "bcc": p.bcc,
                    "from": p.from,
                    "subject": p.subject,
                    "body_excerpt": excerpt(&p.body, 280),
                    "html": p.html.unwrap_or(false),
                }),
            ));
        }

        let opts = SendOptions {
            to: p.to,
            cc: p.cc.unwrap_or_default(),
            bcc: p.bcc.unwrap_or_default(),
            from: p.from,
            subject: p.subject,
            body: p.body,
            html: p.html.unwrap_or(false),
            attachments: Vec::new(),
            send_at: None,
            expires_in: None,
        };

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let id = client
            .save_draft(&opts)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "saved": true, "id": id })))
    }
}
