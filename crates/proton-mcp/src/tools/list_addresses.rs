//! `list_addresses` — list the account's sender addresses.

use rmcp::{tool, tool_router, ErrorData};
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[tool_router(router = list_addresses_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "list_addresses",
        description = "List the account's sender addresses (usable as the `from` field when sending or saving drafts)."
    )]
    pub async fn list_addresses(&self) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let addresses = client.addresses();
        Ok(obj(json!({
            "count": addresses.len(),
            "addresses": to_value(&addresses)?,
        })))
    }
}
