//! `list_conversations` — list conversations in a folder/label (paged).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListConversationsParams {
    /// Folder/label to list. Defaults to `inbox`.
    pub folder: Option<String>,
    /// Restrict to conversations with unread messages.
    pub unread: Option<bool>,
    /// Zero-based page index. Defaults to 0.
    pub page: Option<u32>,
    /// Page size. Defaults to 25.
    pub page_size: Option<u32>,
}

#[tool_router(router = list_conversations_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "list_conversations",
        description = "List conversations in a folder/label (paged). Returns total count and conversation metadata."
    )]
    pub async fn list_conversations(
        &self,
        Parameters(p): Parameters<ListConversationsParams>,
    ) -> Result<Out, ErrorData> {
        let folder = p.folder.unwrap_or_else(|| "inbox".to_string());
        let page = p.page.unwrap_or(0);
        let page_size = p.page_size.unwrap_or(25);
        let unread = p.unread.unwrap_or(false);

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let (total, convs) = client
            .list_conversations(&folder, page, page_size, unread)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "total": total,
            "folder": folder,
            "page": page,
            "page_size": page_size,
            "conversations": to_value(&convs)?,
        })))
    }
}
