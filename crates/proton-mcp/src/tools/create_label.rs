//! `create_label` — create a label or folder (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateLabelParams {
    /// Display name for the label or folder.
    pub name: String,
    /// Hex color (e.g. `#8080FF`). Defaults to `#8080FF`.
    pub color: Option<String>,
    /// Create a folder instead of a label. Defaults to false (label).
    pub folder: Option<bool>,
    /// Parent folder id (folders only).
    pub parent: Option<String>,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = create_label_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "create_label",
        description = "Create a label or folder. Changes account structure: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn create_label(
        &self,
        Parameters(p): Parameters<CreateLabelParams>,
    ) -> Result<Out, ErrorData> {
        let color = p.color.as_deref().unwrap_or("#8080FF");
        let is_folder = p.folder.unwrap_or(false);

        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "create_label",
                json!({ "name": p.name, "color": color, "folder": is_folder, "parent": p.parent }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let label = client
            .create_label(&p.name, color, is_folder, p.parent.as_deref())
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "created": true, "label": to_value(&label)? })))
    }
}
