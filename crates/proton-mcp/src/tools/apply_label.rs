//! `apply_label` — apply a label to messages (confirm-gated).
//!
//! This module owns [`LabelParams`], which is shared with `remove_label`.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LabelParams {
    /// Message ids or free-text references.
    pub references: Vec<String>,
    /// The label id to apply or remove (see `list_labels`).
    pub label_id: String,
    /// Confirm the change. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = apply_label_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "apply_label",
        description = "Apply a label to messages. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn apply_label(
        &self,
        Parameters(p): Parameters<LabelParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "apply_label",
                json!({ "references": p.references, "label_id": p.label_id }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let ids = self.resolve_all(client, &p.references).await?;
        client
            .apply_label(&ids, &p.label_id)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "labeled": ids.len(), "ids": ids, "label_id": p.label_id }),
        ))
    }
}
