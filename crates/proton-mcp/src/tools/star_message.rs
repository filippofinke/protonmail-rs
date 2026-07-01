//! `star_message` — star or unstar messages.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StarMessageParams {
    /// Message ids or free-text references.
    pub references: Vec<String>,
    /// Star (true) or unstar (false).
    pub starred: bool,
}

#[tool_router(router = star_message_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(name = "star_message", description = "Star or unstar messages.")]
    pub async fn star_message(
        &self,
        Parameters(p): Parameters<StarMessageParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let ids = self.resolve_all(client, &p.references).await?;
        client
            .star_messages(&ids, p.starred)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "updated": ids.len(), "ids": ids, "starred": p.starred }),
        ))
    }
}
