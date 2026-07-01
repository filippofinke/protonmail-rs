//! `report_ham` — report messages as not-spam (confirm-gated).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReportHamParams {
    /// Message ids or free-text references to report as not-spam.
    pub references: Vec<String>,
    /// Confirm the action. Without it (and without --allow-writes) a preview is returned.
    pub confirm: Option<bool>,
}

#[tool_router(router = report_ham_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "report_ham",
        description = "Report messages as not-spam (moves them out of Spam to Inbox). Destructive: returns a dry-run preview unless confirm=true or --allow-writes is set."
    )]
    pub async fn report_ham(
        &self,
        Parameters(p): Parameters<ReportHamParams>,
    ) -> Result<Out, ErrorData> {
        if !should_perform(self.state.allow_writes, p.confirm) {
            return Ok(dry_run("report_ham", json!({ "references": p.references })));
        }

        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let ids = self.resolve_all(client, &p.references).await?;
        client.report_ham(&ids).await.map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "reported_ham": ids.len(), "ids": ids })))
    }
}
