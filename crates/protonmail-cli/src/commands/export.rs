//! Export command: dump messages to `.eml` files.

use crate::cli::{Ctx, ExportArgs};
use crate::commands::resume;
use crate::render;
use proton_core::{ExportOptions, Result, SearchOpts};
use std::io::Read;

fn read_ids(args: &ExportArgs) -> Result<Vec<String>> {
    let mut ids: Vec<String> = args.ids.iter().map(|s| s.trim().to_string()).collect();
    if let Some(path) = &args.ids_file {
        let text = if path.as_os_str() == "-" {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            s
        } else {
            std::fs::read_to_string(path)?
        };
        ids.extend(
            text.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(String::from),
        );
    }
    ids.retain(|s| !s.is_empty());
    let mut seen = std::collections::HashSet::new();
    ids.retain(|s| seen.insert(s.clone()));
    Ok(ids)
}

pub async fn run(ctx: &Ctx, args: ExportArgs) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    let eo = ExportOptions {
        concurrency: args.concurrency,
        skip_existing: args.skip_existing,
        attachments: !args.no_attachments,
    };
    let by_id = !args.ids.is_empty() || args.ids_file.is_some();
    let report = if by_id {
        let ids = read_ids(&args)?;
        client.export_messages(&ids, &args.out, &eo).await?
    } else {
        let opts = SearchOpts {
            folder: Some(args.folder.clone()),
            limit: Some(args.max),
            keyword: args.keyword.clone(),
            from: args.from.clone(),
            to: args.to.clone(),
            subject: args.subject.clone(),
            after: args.after.clone(),
            before: args.before.clone(),
            unread: args.unread,
        };
        client.export_search(&opts, &args.out, &eo).await?
    };
    let out = args.out.display().to_string();
    if ctx.json {
        render::json_out(&serde_json::json!({
            "exported": report.exported,
            "skipped": report.skipped,
            "failed": report.failed.iter().map(|(id, e)| serde_json::json!({"id": id, "error": e})).collect::<Vec<_>>(),
            "out": out,
        }));
    } else {
        println!(
            "Exported {} message(s) to {out}{}",
            report.exported,
            if report.skipped > 0 {
                format!(" ({} already present, skipped)", report.skipped)
            } else {
                String::new()
            }
        );
        for (id, e) in &report.failed {
            eprintln!("failed: {id}: {e}");
        }
    }
    if !report.failed.is_empty() {
        return Err(proton_core::Error::Other(format!(
            "{} message(s) failed to export",
            report.failed.len()
        )));
    }
    Ok(())
}
