//! `remove_label` — remove a label from messages (confirm-gated).
//!
//! Reuses [`LabelParams`] from the `apply_label` module.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;
use crate::tools::apply_label::LabelParams;

#[tool_router(router = remove_label_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "remove_label",
        description = "Remove a label from messages. Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn remove_label(
        &self,
        Parameters(p): Parameters<LabelParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run(
                "remove_label",
                json!({ "references": p.references, "label_id": p.label_id }),
            ));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let ids = self.resolve_all(client, &p.references).await?;
        client
            .remove_label(&ids, &p.label_id)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(
            json!({ "unlabeled": ids.len(), "ids": ids, "label_id": p.label_id }),
        ))
    }
}
