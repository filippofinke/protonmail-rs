//! Message commands.

use crate::cli::Ctx;
use crate::cli::{LabelAction, MessagesCmd, SearchArgs, SendArgs};
use crate::commands::{read_body, resolve_all, resume};
use crate::render;
use proton_core::{Client, Result, SearchOpts, SendOptions};

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

pub(crate) fn send_options(a: SendArgs, body: String) -> SendOptions {
    SendOptions {
        to: a.to,
        cc: a.cc,
        bcc: a.bcc,
        from: a.from,
        subject: a.subject,
        body,
        html: a.html,
        attachments: a.attach,
        send_at: a.send_at,
        expires_in: a.expires,
    }
}

pub async fn run(ctx: &Ctx, cmd: MessagesCmd) -> Result<()> {
    match cmd {
        MessagesCmd::List {
            folder,
            page,
            page_size,
            unread,
            cached,
        } => {
            let client = resume(&ctx.profile).await?;
            if cached {
                let msgs = client.cached_messages(&folder, unread, page_size)?;
                render::messages_list(ctx.json, msgs.len() as u32, &msgs);
            } else {
                let (total, msgs) = client
                    .list_messages(&folder, page, page_size, unread)
                    .await?;
                render::messages_list(ctx.json, total, &msgs);
            }
            Ok(())
        }
        MessagesCmd::Search(args) => {
            let client = resume(&ctx.profile).await?;
            let opts = search_opts(args);
            let msgs = client.search_messages(&opts).await?;
            render::messages_list(ctx.json, msgs.len() as u32, &msgs);
            Ok(())
        }
        MessagesCmd::Read {
            reference,
            format,
            body_only,
            output,
        } => {
            let client = resume(&ctx.profile).await?;
            let id = client.resolve_ref(&reference).await?;
            let msg = client.read_message(&id).await?;
            if let Some(path) = output {
                std::fs::write(&path, &msg.body)?;
                if !ctx.json {
                    println!("Wrote {} bytes to {}", msg.body.len(), path.display());
                }
            } else {
                render::full_message(ctx.json, &msg, format, body_only);
            }
            Ok(())
        }
        MessagesCmd::Send(args) => {
            let client = resume(&ctx.profile).await?;
            let body = read_body(&args.body)?;
            let eo_password = args.eo_password.clone();
            let eo_hint = args.eo_hint.clone();
            let opts = send_options(args, body);
            let id = match &eo_password {
                Some(pw) => client.send_eo(&opts, pw, eo_hint.as_deref()).await?,
                None => client.send(&opts).await?,
            };
            render::sent(ctx.json, &id);
            Ok(())
        }
        MessagesCmd::Reply {
            reference,
            all,
            from,
            body,
            attach,
        } => {
            let client = resume(&ctx.profile).await?;
            let body = read_body(&body)?;
            let opts = SendOptions {
                from,
                body,
                attachments: attach,
                ..Default::default()
            };
            let id = client.reply(&reference, all, &opts).await?;
            render::sent(ctx.json, &id);
            Ok(())
        }
        MessagesCmd::Forward {
            reference,
            to,
            from,
            body,
            attach,
        } => {
            let client = resume(&ctx.profile).await?;
            let body = read_body(&body)?;
            let opts = SendOptions {
                to,
                from,
                body,
                attachments: attach,
                ..Default::default()
            };
            let id = client.forward(&reference, &opts).await?;
            render::sent(ctx.json, &id);
            Ok(())
        }
        MessagesCmd::CancelSend { reference } => {
            let client = resume(&ctx.profile).await?;
            client.cancel_send(&reference).await?;
            render::action_result(ctx.json, "cancel-send", std::slice::from_ref(&reference));
            Ok(())
        }
        MessagesCmd::Move { references, dest } => {
            organize(ctx, &references, "move", move |c, ids| async move {
                c.move_messages(&ids, &dest).await
            })
            .await
        }
        MessagesCmd::Trash { references } => {
            organize(ctx, &references, "trash", |c, ids| async move {
                c.trash_messages(&ids).await
            })
            .await
        }
        MessagesCmd::Delete { references } => {
            organize(ctx, &references, "delete", |c, ids| async move {
                c.delete_messages(&ids).await
            })
            .await
        }
        MessagesCmd::Mark { state, references } => {
            let read = state.as_bool();
            organize(ctx, &references, "mark", move |c, ids| async move {
                c.mark_messages_read(&ids, read).await
            })
            .await
        }
        MessagesCmd::Star { references } => {
            organize(ctx, &references, "star", |c, ids| async move {
                c.star_messages(&ids, true).await
            })
            .await
        }
        MessagesCmd::Unstar { references } => {
            organize(ctx, &references, "unstar", |c, ids| async move {
                c.star_messages(&ids, false).await
            })
            .await
        }
        MessagesCmd::Spam { references } => {
            organize(ctx, &references, "spam", |c, ids| async move {
                c.report_spam(&ids).await
            })
            .await
        }
        MessagesCmd::Ham { references } => {
            organize(ctx, &references, "ham", |c, ids| async move {
                c.report_ham(&ids).await
            })
            .await
        }
        MessagesCmd::Unsubscribe { reference } => {
            let client = resume(&ctx.profile).await?;
            let id = client.resolve_ref(&reference).await?;
            client.unsubscribe(&id).await?;
            render::action_result(ctx.json, "unsubscribed", std::slice::from_ref(&id));
            Ok(())
        }
        MessagesCmd::Empty { folder } => {
            let client = resume(&ctx.profile).await?;
            client.empty_folder(&folder).await?;
            render::action_result(ctx.json, "emptied", std::slice::from_ref(&folder));
            Ok(())
        }
        MessagesCmd::Label {
            action,
            label_id,
            references,
        } => {
            let name = match action {
                LabelAction::Add => "label",
                LabelAction::Rm => "unlabel",
            };
            organize(ctx, &references, name, move |c, ids| async move {
                match action {
                    LabelAction::Add => c.apply_label(&ids, &label_id).await,
                    LabelAction::Rm => c.remove_label(&ids, &label_id).await,
                }
            })
            .await
        }
        MessagesCmd::Undelete { ids } => {
            let client = resume(&ctx.profile).await?;
            client.undelete_messages(&ids).await?;
            render::action_result(ctx.json, "undeleted", &ids);
            Ok(())
        }
        MessagesCmd::Receipt { reference } => {
            let client = resume(&ctx.profile).await?;
            let id = client.resolve_ref(&reference).await?;
            client.send_read_receipt(&id).await?;
            render::action_result(ctx.json, "receipt sent", std::slice::from_ref(&id));
            Ok(())
        }
    }
}

/// Resolve references, run a bulk action over the resulting IDs, and report.
async fn organize<F, Fut>(ctx: &Ctx, refs: &[String], action: &str, op: F) -> Result<()>
where
    F: FnOnce(Client, Vec<String>) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let client = resume(&ctx.profile).await?;
    let ids = resolve_all(&client, refs).await?;
    op(client, ids.clone()).await?;
    render::action_result(ctx.json, action, &ids);
    Ok(())
}
