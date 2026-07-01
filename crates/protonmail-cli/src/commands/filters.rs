//! Sieve filter commands.

use crate::cli::{Ctx, FiltersCmd};
use crate::commands::{read_body, resume};
use crate::render;
use proton_core::Result;

pub async fn run(ctx: &Ctx, cmd: FiltersCmd) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    match cmd {
        FiltersCmd::List => {
            let filters = client.list_filters().await?;
            render::filters_list(ctx.json, &filters);
        }
        FiltersCmd::Create { name, sieve } => {
            let script = read_body(&sieve)?;
            let f = client.create_filter(&name, &script).await?;
            render::filter_created(ctx.json, &f);
        }
        FiltersCmd::Check { sieve } => {
            let script = read_body(&sieve)?;
            client.check_filter(&script).await?;
            if ctx.json {
                render::json_out(&serde_json::json!({ "valid": true }));
            } else {
                println!("Sieve script is valid.");
            }
        }
        FiltersCmd::Delete { id } => {
            client.delete_filter(&id).await?;
            render::action_result(ctx.json, "filter deleted", &[id]);
        }
        FiltersCmd::Enable { id } => {
            client.enable_filter(&id).await?;
            render::action_result(ctx.json, "filter enabled", &[id]);
        }
        FiltersCmd::Disable { id } => {
            client.disable_filter(&id).await?;
            render::action_result(ctx.json, "filter disabled", &[id]);
        }
    }
    Ok(())
}
