//! Local encrypted-search commands: build index, query it.

use crate::cli::Ctx;
use crate::commands::resume;
use crate::render;
use proton_core::Result;
use serde_json::json;

pub async fn index(ctx: &Ctx, folder: String, max_pages: u32, page_size: u32) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    let n = client.index_folder(&folder, max_pages, page_size).await?;
    if ctx.json {
        render::json_out(&json!({ "indexed": n }));
    } else {
        println!("Indexed {n} message(s) for local search.");
    }
    Ok(())
}

pub async fn search(ctx: &Ctx, query: String, limit: u32) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    let msgs = client.search_local(&query, limit)?;
    render::messages_list(ctx.json, msgs.len() as u32, &msgs);
    Ok(())
}
