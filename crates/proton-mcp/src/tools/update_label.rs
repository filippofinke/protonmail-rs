//! `update_label` — rename / recolor / reparent a label or folder (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateLabelParams {
    /// The label or folder id to update.
    pub id: String,
    /// New display name.
    pub name: Option<String>,
    /// New hex color (e.g. `#8080FF`).
    pub color: Option<String>,
    /// New parent folder id (folders only).
    pub parent: Option<String>,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = update_label_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "update_label",
        description = "Rename, recolor, or reparent a label or folder. Changes account structure: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn update_label(
        &self,
        Parameters(p): Parameters<UpdateLabelParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "update_label",
                json!({ "id": p.id, "name": p.name, "color": p.color, "parent": p.parent }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .update_label(
                &p.id,
                p.name.as_deref(),
                p.color.as_deref(),
                p.parent.as_deref(),
            )
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "updated": true, "id": p.id })))
    }
}
