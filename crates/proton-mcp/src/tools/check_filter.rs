//! `check_filter` — validate a Sieve script without creating a filter.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheckFilterParams {
    /// The Sieve script to validate.
    pub sieve: String,
}

#[tool_router(router = check_filter_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "check_filter",
        description = "Validate a Sieve filter script server-side without creating a filter. Errors if the script is invalid."
    )]
    pub async fn check_filter(
        &self,
        Parameters(p): Parameters<CheckFilterParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        client
            .check_filter(&p.sieve)
            .await
            .map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "valid": true })))
    }
}
