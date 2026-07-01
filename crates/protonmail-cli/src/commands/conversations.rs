//! Conversation commands.
//!
//! Conversation references are treated as literal conversation IDs: the core
//! `resolve_ref` helper resolves to *message* IDs, and there is no
//! conversation-level resolver, so free-text resolution is not available here.

use crate::cli::Ctx;
use crate::cli::{ConversationsCmd, SearchArgs};
use crate::render;
use proton_core::{Result, SearchOpts};

fn search_opts(a: SearchArgs) -> SearchOpts {
    SearchOpts {
        keyword: a.keyword,
        from: a.from,
        to: a.to,
        subject: a.subject,
        after: a.after,
        before: a.before,
        folder: a.folder,
        unread: a.unread,
        limit: a.limit,
    }
}

pub async fn run(ctx: &Ctx, cmd: ConversationsCmd) -> Result<()> {
    let client = crate::commands::resume(&ctx.profile).await?;
    match cmd {
        ConversationsCmd::List {
            folder,
            page,
            page_size,
            unread,
        } => {
            let (total, convs) = client
                .list_conversations(&folder, page, page_size, unread)
                .await?;
            render::conversations_list(ctx.json, total, &convs);
        }
        ConversationsCmd::Search(args) => {
            let convs = client.search_conversations(&search_opts(args)).await?;
            render::conversations_list(ctx.json, convs.len() as u32, &convs);
        }
        ConversationsCmd::Read { id } => {
            let (conv, msgs) = client.read_conversation(&id).await?;
            render::conversation_read(ctx.json, &conv, &msgs);
        }
        ConversationsCmd::Move { ids, dest } => {
            client.move_conversations(&ids, &dest).await?;
            render::action_result(ctx.json, "move", &ids);
        }
        ConversationsCmd::Trash { ids } => {
            client.trash_conversations(&ids).await?;
            render::action_result(ctx.json, "trash", &ids);
        }
        ConversationsCmd::Mark { state, folder, ids } => {
            client
                .mark_conversations_read(&ids, state.as_bool(), &folder)
                .await?;
            render::action_result(ctx.json, "mark", &ids);
        }
        ConversationsCmd::Star { ids } => {
            client.star_conversations(&ids, true).await?;
            render::action_result(ctx.json, "star", &ids);
        }
        ConversationsCmd::Unstar { ids } => {
            client.star_conversations(&ids, false).await?;
            render::action_result(ctx.json, "unstar", &ids);
        }
        ConversationsCmd::Snooze {
            ids,
            until,
            relative,
        } => {
            let ts = match (until, relative) {
                (Some(t), _) => t,
                (None, Some(d)) => now_unix() + parse_duration_secs(&d)?,
                (None, None) => {
                    return Err(proton_core::Error::Other(
                        "provide --until <unix> or --in <duration>".into(),
                    ))
                }
            };
            client.snooze_conversations(&ids, ts).await?;
            render::action_result(ctx.json, "snoozed", &ids);
        }
        ConversationsCmd::Unsnooze { ids } => {
            client.unsnooze_conversations(&ids).await?;
            render::action_result(ctx.json, "unsnoozed", &ids);
        }
    }
    Ok(())
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Parse `30m`, `2h`, `1d`, `45s` (or bare seconds) into seconds.
fn parse_duration_secs(s: &str) -> Result<i64> {
    let s = s.trim();
    let split = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
    let (num, unit) = s.split_at(split);
    let n: i64 = num
        .parse()
        .map_err(|_| proton_core::Error::Other(format!("bad duration: {s}")))?;
    let mult = match unit {
        "" | "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        other => {
            return Err(proton_core::Error::Other(format!(
                "bad duration unit: {other}"
            )))
        }
    };
    Ok(n * mult)
}
