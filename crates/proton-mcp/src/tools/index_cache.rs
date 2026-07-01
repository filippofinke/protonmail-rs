//! `index_cache` — build the local encrypted-search index over a folder's bodies.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IndexCacheParams {
    /// Folder/label to index (e.g. inbox, sent, all). Defaults to all mail.
    pub folder: Option<String>,
    /// Maximum pages to fetch. Defaults to 3.
    pub max_pages: Option<u32>,
    /// Page size. Defaults to 50.
    pub page_size: Option<u32>,
}

#[tool_router(router = index_cache_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "index_cache",
        description = "Build the local encrypted-search index by decrypting and indexing message bodies in a folder. Populates the index used by search_local."
    )]
    pub async fn index_cache(
        &self,
        Parameters(p): Parameters<IndexCacheParams>,
    ) -> Result<Out, ErrorData> {
        let folder = p.folder.unwrap_or_else(|| "all".to_string());
        let max_pages = p.max_pages.unwrap_or(3);
        let page_size = p.page_size.unwrap_or(50);

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let indexed = client
            .index_folder(&folder, max_pages, page_size)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "indexed": indexed, "folder": folder })))
    }
}
