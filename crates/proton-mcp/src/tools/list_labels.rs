//! `list_labels` — list the account's labels and folders.

use rmcp::{tool, tool_router, ErrorData};
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[tool_router(router = list_labels_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "list_labels",
        description = "List the account's labels and folders (with ids usable by apply_label/remove_label/move_message)."
    )]
    pub async fn list_labels(&self) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let labels = client.list_labels().await.map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "count": labels.len(),
            "labels": to_value(&labels)?,
        })))
    }
}
