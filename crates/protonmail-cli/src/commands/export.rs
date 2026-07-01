//! Export command: dump a folder to `.eml` files.

use crate::cli::Ctx;
use crate::commands::resume;
use crate::render;
use proton_core::Result;
use std::path::PathBuf;

pub async fn run(ctx: &Ctx, folder: String, out: PathBuf, max: u32) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    let n = client.export_folder(&folder, &out, max).await?;
    if ctx.json {
        render::json_out(&serde_json::json!({ "exported": n, "out": out.display().to_string() }));
    } else {
        println!("Exported {n} message(s) to {}", out.display());
    }
    Ok(())
}
