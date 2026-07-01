//! Draft commands: list, save, edit, delete.

use crate::cli::{Ctx, DraftsCmd};
use crate::commands::messages::send_options;
use crate::commands::{read_body, resume};
use crate::render;
use proton_core::Result;

pub async fn run(ctx: &Ctx, cmd: DraftsCmd) -> Result<()> {
    let client = resume(&ctx.profile).await?;
    match cmd {
        DraftsCmd::List { page, page_size } => {
            let (total, msgs) = client.list_drafts(page, page_size).await?;
            render::messages_list(ctx.json, total, &msgs);
        }
        DraftsCmd::Save(args) => {
            let body = read_body(&args.body)?;
            let opts = send_options(args, body);
            let id = client.save_draft(&opts).await?;
            render::sent(ctx.json, &id);
        }
        DraftsCmd::Edit { id, args } => {
            let body = read_body(&args.body)?;
            let opts = send_options(args, body);
            client.update_draft(&id, &opts).await?;
            render::action_result(ctx.json, "draft updated", &[id]);
        }
        DraftsCmd::Delete { ids } => {
            client.delete_draft(&ids).await?;
            render::action_result(ctx.json, "draft deleted", &ids);
        }
    }
    Ok(())
}
