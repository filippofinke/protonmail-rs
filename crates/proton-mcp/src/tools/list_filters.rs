//! `list_filters` — list the account's mail filters.

use rmcp::{tool, tool_router, ErrorData};
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[tool_router(router = list_filters_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "list_filters",
        description = "List the account's mail filters (sieve rules)."
    )]
    pub async fn list_filters(&self) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        // `proton_core::Filter` derives only `Deserialize`, so build the JSON
        // from its public fields rather than serializing it directly.
        let filters = client.list_filters().await.map_err(|e| self.map_err(e))?;
        let filters: Vec<_> = filters
            .iter()
            .map(|f| {
                json!({
                    "id": f.id,
                    "name": f.name,
                    "status": f.status,
                    "version": f.version,
                })
            })
            .collect();
        Ok(obj(json!({
            "count": filters.len(),
            "filters": filters,
        })))
    }
}
