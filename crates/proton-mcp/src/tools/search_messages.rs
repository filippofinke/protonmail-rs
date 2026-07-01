//! `search_messages` — search messages within a folder.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use proton_core::SearchOpts;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchMessagesParams {
    /// Free-text keyword to match.
    pub keyword: Option<String>,
    /// Filter by sender address.
    pub from: Option<String>,
    /// Filter by recipient address.
    pub to: Option<String>,
    /// Filter by subject.
    pub subject: Option<String>,
    /// Only messages after this date (YYYY-MM-DD).
    pub after: Option<String>,
    /// Only messages before this date (YYYY-MM-DD).
    pub before: Option<String>,
    /// Folder/label to search within. Defaults to all mail.
    pub folder: Option<String>,
    /// Maximum number of results. Defaults to 25.
    pub limit: Option<u32>,
}

#[tool_router(router = search_messages_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "search_messages",
        description = "Search messages by keyword/sender/recipient/subject/date range within a folder."
    )]
    pub async fn search_messages(
        &self,
        Parameters(p): Parameters<SearchMessagesParams>,
    ) -> Result<Out, ErrorData> {
        let opts = SearchOpts {
            keyword: p.keyword,
            from: p.from,
            to: p.to,
            subject: p.subject,
            after: p.after,
            before: p.before,
            folder: p.folder,
            unread: false,
            limit: p.limit,
        };

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let msgs = client
            .search_messages(&opts)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "count": msgs.len(),
            "messages": to_value(&msgs)?,
        })))
    }
}
