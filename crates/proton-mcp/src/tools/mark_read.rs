//! `mark_read` — mark messages read or unread.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MarkReadParams {
    /// Message ids or free-text references.
    pub references: Vec<String>,
    /// Mark as read (true) or unread (false).
    pub read: bool,
}

#[tool_router(router = mark_read_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(name = "mark_read", description = "Mark messages read or unread.")]
    pub async fn mark_read(
        &self,
        Parameters(p): Parameters<MarkReadParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let ids = self.resolve_all(client, &p.references).await?;
        client
            .mark_messages_read(&ids, p.read)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "updated": ids.len(), "ids": ids, "read": p.read }),
        ))
    }
}
