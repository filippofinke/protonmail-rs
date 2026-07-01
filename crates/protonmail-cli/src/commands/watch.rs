//! Watch command: continuously sync the cache from the event stream.

use crate::cli::Ctx;
use crate::commands::resume;
use proton_core::Result;
use std::time::Duration;

pub async fn run(ctx: &Ctx, interval: u64, folder: Option<String>) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    eprintln!("watching (every {interval}s; Ctrl-C to stop)…");
    loop {
        let r = client.sync().await?;
        if let Some(f) = &folder {
            let _ = client.cache_folder(f, 4, 50).await;
            let _ = client.index_folder(f, 2, 50).await;
        }
        let tag = if r.initialized { " (initialized)" } else { "" };
        println!("[sync] +{} ~{} -{}{tag}", r.created, r.updated, r.deleted);
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}
