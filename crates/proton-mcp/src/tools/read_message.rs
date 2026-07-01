//! `read_message` — read and decrypt a single message.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadMessageParams {
    /// Message id, or free text that uniquely identifies a message.
    pub reference: String,
    /// `full` (default) returns the whole body; `excerpt` returns a short preview.
    pub format: Option<String>,
}

#[tool_router(router = read_message_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "read_message",
        description = "Read and decrypt a message. Returns the signature verdict, body, attachments, and a warning when the signature is not verified."
    )]
    pub async fn read_message(
        &self,
        Parameters(p): Parameters<ReadMessageParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let id = self.resolve(client, &p.reference).await?;
        let full = client
            .read_message(&id)
            .await
            .map_err(|e| self.map_err(e))?;

        let want_excerpt = p.format.as_deref() == Some("excerpt");
        let body = if want_excerpt {
            excerpt(&full.body, 2000)
        } else {
            full.body.clone()
        };
        let warning = signature_warning(full.verdict);

        Ok(obj(json!({
            "id": id,
            "meta": to_value(&full.meta)?,
            "verdict": to_value(&full.verdict)?,
            "mime_type": full.mime_type,
            "body": body,
            "excerpt": excerpt(&full.body, 280),
            "attachments": to_value(&full.attachments)?,
            "warning": warning,
        })))
    }
}
