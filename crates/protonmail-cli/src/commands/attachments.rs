//! Attachment commands.

use crate::cli::AttachmentsCmd;
use crate::cli::Ctx;
use crate::commands::resume;
use crate::render;
use proton_core::{Error, Result};
use serde_json::json;
use std::path::{Path, PathBuf};

pub async fn run(ctx: &Ctx, cmd: AttachmentsCmd) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    match cmd {
        AttachmentsCmd::List {
            message,
            include_inline,
        } => {
            let msg_id = client.resolve_ref(&message).await?;
            let atts = client.list_attachments(&msg_id, include_inline).await?;
            render::attachments_list(ctx.json, &atts);
            Ok(())
        }
        AttachmentsCmd::Download {
            message,
            attachment,
            output_dir,
            all,
            include_inline,
        } => {
            let msg_id = client.resolve_ref(&message).await?;
            let dir = output_dir.unwrap_or_else(|| PathBuf::from("."));
            std::fs::create_dir_all(&dir)?;

            let files: Vec<(String, Vec<u8>)> = if all {
                client
                    .download_all_attachments(&msg_id, include_inline)
                    .await?
            } else if let Some(att_id) = attachment {
                vec![client.download_attachment(&msg_id, &att_id).await?]
            } else {
                return Err(Error::Other(
                    "specify an attachment ID or use --all".to_string(),
                ));
            };

            let mut written = Vec::with_capacity(files.len());
            for (name, bytes) in &files {
                let path = unique_path(&dir, name);
                std::fs::write(&path, bytes)?;
                written.push(path.to_string_lossy().to_string());
            }

            if ctx.json {
                render::json_out(&json!({
                    "status": "ok",
                    "count": written.len(),
                    "files": written,
                }));
            } else {
                println!("Downloaded {} file(s):", written.len());
                for f in &written {
                    println!("  {f}");
                }
            }
            Ok(())
        }
    }
}

/// Avoid clobbering existing files by appending ` (n)` before the extension.
fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let safe = if name.is_empty() { "attachment" } else { name };
    let candidate = dir.join(safe);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(safe);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(safe);
    let ext = path.extension().and_then(|s| s.to_str());
    for n in 1..10_000 {
        let fname = match ext {
            Some(e) => format!("{stem} ({n}).{e}"),
            None => format!("{stem} ({n})"),
        };
        let p = dir.join(fname);
        if !p.exists() {
            return p;
        }
    }
    candidate
}
