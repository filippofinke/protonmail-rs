//! `list_attachments` — list a message's attachments.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAttachmentsParams {
    /// Message id or free text identifying a message.
    pub message_ref: String,
    /// Include inline attachments (e.g. embedded images). Defaults to false.
    pub include_inline: Option<bool>,
}

#[tool_router(router = list_attachments_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "list_attachments",
        description = "List a message's attachments (inline ones excluded unless include_inline=true)."
    )]
    pub async fn list_attachments(
        &self,
        Parameters(p): Parameters<ListAttachmentsParams>,
    ) -> Result<Out, ErrorData> {
        let include_inline = p.include_inline.unwrap_or(false);

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let id = self.resolve(client, &p.message_ref).await?;
        let atts = client
            .list_attachments(&id, include_inline)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "message_id": id,
            "count": atts.len(),
            "attachments": to_value(&atts)?,
        })))
    }
}
