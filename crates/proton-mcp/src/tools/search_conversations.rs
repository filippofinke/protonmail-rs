//! `search_conversations` — search conversations (threads) within a folder.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use proton_core::SearchOpts;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchConversationsParams {
    /// Free-text keyword to match.
    pub keyword: Option<String>,
    /// Filter by sender address.
    pub from: Option<String>,
    /// Filter by recipient address.
    pub to: Option<String>,
    /// Filter by subject.
    pub subject: Option<String>,
    /// Only conversations after this date (YYYY-MM-DD).
    pub after: Option<String>,
    /// Only conversations before this date (YYYY-MM-DD).
    pub before: Option<String>,
    /// Folder/label to search within. Defaults to all mail.
    pub folder: Option<String>,
    /// Only conversations with unread messages.
    pub unread: Option<bool>,
    /// Maximum number of results. Defaults to 25.
    pub limit: Option<u32>,
}

#[tool_router(router = search_conversations_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "search_conversations",
        description = "Search conversations (threads) by keyword/sender/recipient/subject/date range within a folder."
    )]
    pub async fn search_conversations(
        &self,
        Parameters(p): Parameters<SearchConversationsParams>,
    ) -> Result<Out, ErrorData> {
        let opts = SearchOpts {
            keyword: p.keyword,
            from: p.from,
            to: p.to,
            subject: p.subject,
            after: p.after,
            before: p.before,
            folder: p.folder,
            unread: p.unread.unwrap_or(false),
            limit: p.limit,
        };

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let convs = client
            .search_conversations(&opts)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "count": convs.len(),
            "conversations": to_value(&convs)?,
        })))
    }
}
