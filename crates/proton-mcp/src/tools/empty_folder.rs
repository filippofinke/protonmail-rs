//! `empty_folder` — permanently delete every message in a folder (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmptyFolderParams {
    /// Folder to empty (e.g. `trash`, `spam`, or a label id).
    pub folder: String,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = empty_folder_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "empty_folder",
        description = "Permanently delete ALL messages in a folder (e.g. trash, spam). IRREVERSIBLE — emptied messages cannot be recovered. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn empty_folder(
        &self,
        Parameters(p): Parameters<EmptyFolderParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "empty_folder",
                json!({
                    "folder": p.folder,
                    "warning": "Permanently deletes EVERY message in this folder; cannot be undone.",
                }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .empty_folder(&p.folder)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "emptied": p.folder })))
    }
}
