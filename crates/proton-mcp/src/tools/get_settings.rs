//! `get_settings` — read the account's mail settings.

use rmcp::{tool, tool_router, ErrorData};
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[tool_router(router = get_settings_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "get_settings",
        description = "Read the account's mail settings (signing, key attachment, display, etc.) as raw JSON."
    )]
    pub async fn get_settings(&self) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let settings = client.mail_settings().await.map_err(|e| self.map_err(e))?;
        Ok(obj(json!({ "settings": settings })))
    }
}
