//! `list_contact_emails` — list contact email addresses (optionally filtered).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListContactEmailsParams {
    /// Optional email to filter by (returns only matching contact emails).
    pub email: Option<String>,
}

#[tool_router(router = list_contact_emails_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "list_contact_emails",
        description = "List contact email addresses, optionally filtered by a specific email. Returns name + email + contact id."
    )]
    pub async fn list_contact_emails(
        &self,
        Parameters(p): Parameters<ListContactEmailsParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let emails = client
            .list_contact_emails(p.email.as_deref())
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({
            "count": emails.len(),
            "emails": to_value(&emails)?,
        })))
    }
}
