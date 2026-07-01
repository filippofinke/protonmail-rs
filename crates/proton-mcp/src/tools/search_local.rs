//! `search_local` — full-text search the local encrypted index.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchLocalParams {
    /// Full-text query to match against the local index.
    pub query: String,
    /// Maximum number of results. Defaults to 25.
    pub limit: Option<u32>,
}

#[tool_router(router = search_local_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "search_local",
        description = "Full-text search the local encrypted index (decrypted bodies; private + offline). Run `sync_cache` first to populate the index."
    )]
    pub async fn search_local(
        &self,
        Parameters(p): Parameters<SearchLocalParams>,
    ) -> Result<Out, ErrorData> {
        let limit = p.limit.unwrap_or(25);

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let msgs = client
            .search_local(&p.query, limit)
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "count": msgs.len(),
            "messages": to_value(&msgs)?,
        })))
    }
}
