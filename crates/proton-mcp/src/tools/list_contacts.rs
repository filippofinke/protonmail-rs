//! `list_contacts` — list the account's contacts (paged).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListContactsParams {
    /// Zero-based page index. Defaults to 0.
    pub page: Option<u32>,
    /// Page size. Defaults to 25.
    pub page_size: Option<u32>,
}

#[tool_router(router = list_contacts_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "list_contacts",
        description = "List the account's contacts (paged). Returns total count and contact records."
    )]
    pub async fn list_contacts(
        &self,
        Parameters(p): Parameters<ListContactsParams>,
    ) -> Result<Out, ErrorData> {
        let page = p.page.unwrap_or(0);
        let page_size = p.page_size.unwrap_or(25);

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let (total, contacts) = client
            .list_contacts(page, page_size)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "total": total,
            "page": page,
            "page_size": page_size,
            "contacts": to_value(&contacts)?,
        })))
    }
}
