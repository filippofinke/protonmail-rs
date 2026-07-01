//! `download_attachment` — download and decrypt one attachment to disk.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;
use crate::server::ProtonMail;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DownloadAttachmentParams {
    /// Message id or free text identifying the message.
    pub message_ref: String,
    /// The attachment id (see `list_attachments`).
    pub attachment_id: String,
    /// Directory to write the decrypted attachment into.
    pub dest_dir: String,
}

#[tool_router(router = download_attachment_router, vis = "pub(crate)")]
impl ProtonMail {
    #[tool(
        name = "download_attachment",
        description = "Download and decrypt one attachment, writing it into dest_dir. Returns the written file path."
    )]
    pub async fn download_attachment(
        &self,
        Parameters(p): Parameters<DownloadAttachmentParams>,
    ) -> Result<Out, ErrorData> {
        let mut guard = self.state.client.lock().await;
        self.ensure(&mut guard).await?;
        let client = guard.as_ref().expect("client present");

        let message_id = self.resolve(client, &p.message_ref).await?;
        let (filename, bytes) = client
            .download_attachment(&message_id, &p.attachment_id)
            .await
            .map_err(|e| self.map_err(e))?;

        let dir = std::path::Path::new(&p.dest_dir);
        std::fs::create_dir_all(dir)
            .map_err(|e| ErrorData::internal_error(format!("create dir: {e}"), None))?;
        // Use only the file name component to avoid path traversal from the server name.
        let safe_name = std::path::Path::new(&filename)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "attachment.bin".to_string());
        let path = dir.join(&safe_name);
        let written = bytes.len();
        std::fs::write(&path, &bytes)
            .map_err(|e| ErrorData::internal_error(format!("write file: {e}"), None))?;

        Ok(obj(json!({
            "saved": true,
            "filename": safe_name,
            "path": path.to_string_lossy(),
            "bytes_written": written,
        })))
    }
}
