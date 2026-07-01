//! `sync_cache` — apply incremental events into the local cache (optionally backfill a folder).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncCacheParams {
    /// Optional folder/label to backfill into the local cache after syncing (e.g. inbox, sent, archive, or a label id).
    pub backfill: Option<String>,
}

#[tool_router(router = sync_cache_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "sync_cache",
        description = "Apply incremental events into the local encrypted cache; optionally backfill a folder. Populates the index used by search_local."
    )]
    pub async fn sync_cache(
        &self,
        Parameters(p): Parameters<SyncCacheParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let report = client.sync().await.map_err(|e| self.map_err(e))?;

        let backfilled = match &p.backfill {
            Some(folder) => Some(
                client
                    .cache_folder(folder, 4, 50)
                    .await
                    .map_err(|e| self.map_err(e))?,
            ),
            None => None,
        };

        Ok(obj(json!({
            "initialized": report.initialized,
            "created": report.created,
            "updated": report.updated,
            "deleted": report.deleted,
            "event_id": report.event_id,
            "backfilled": backfilled,
        })))
    }
}
