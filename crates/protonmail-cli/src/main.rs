//! protonmail-cli — a clap v4 command-line front-end over `proton-core`.

mod cli;
mod commands;
mod hv;
mod render;

use clap::Parser;
use cli::{Cli, Command, Ctx};
use proton_core::Result;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    // Logs go to stderr so stdout stays clean for text/JSON output.
    // -v debug, -vv trace(core), -vvv trace(all); RUST_LOG overrides.
    proton_core::init_tracing(cli.verbose);

    if let Err(e) = dispatch(cli).await {
        eprintln!("error: {e}");
        std::process::exit(e.exit_code());
    }
}

async fn dispatch(cli: Cli) -> Result<()> {
    let ctx: Ctx = (&cli).into();
    match cli.command {
        Command::Login => commands::auth::login(&ctx).await,
        Command::Logout => commands::auth::logout(&ctx).await,
        Command::Whoami => commands::auth::whoami(&ctx).await,
        Command::Messages { cmd } => commands::messages::run(&ctx, cmd).await,
        Command::Conversations { cmd } => commands::conversations::run(&ctx, cmd).await,
        Command::Attachments { cmd } => commands::attachments::run(&ctx, cmd).await,
        Command::Drafts { cmd } => commands::drafts::run(&ctx, cmd).await,
        Command::Filters { cmd } => commands::filters::run(&ctx, cmd).await,
        Command::Contacts { cmd } => commands::contacts::run(&ctx, cmd).await,
        Command::Addresses { cmd } => commands::contacts::addresses(&ctx, cmd).await,
        Command::Settings { cmd } => commands::contacts::settings(&ctx, cmd).await,
        Command::Counts { conversations } => commands::contacts::counts(&ctx, conversations).await,
        Command::Export { folder, out, max } => commands::export::run(&ctx, folder, out, max).await,
        Command::Watch { interval, folder } => commands::watch::run(&ctx, interval, folder).await,
        Command::Index {
            folder,
            max_pages,
            page_size,
        } => commands::search::index(&ctx, folder, max_pages, page_size).await,
        Command::Search { query, limit } => commands::search::search(&ctx, query, limit).await,
        Command::Sync {
            backfill,
            max_pages,
            page_size,
        } => commands::sync::run(&ctx, backfill, max_pages, page_size).await,
        Command::Labels { cmd } => commands::labels::run(&ctx, cmd).await,
    }
}

#[cfg(test)]
mod tests {
    use super::cli::{Cli, Command, ConversationsCmd, MessagesCmd, ReadFormat, ReadState};
    use clap::{CommandFactory, Parser};

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("args should parse")
    }

    #[test]
    fn verify_clap_command() {
        // Guards against derive macro misconfiguration.
        Cli::command().debug_assert();
    }

    #[test]
    fn send_with_stdin_body() {
        let cli = parse(&[
            "protonmail-cli",
            "messages",
            "send",
            "--to",
            "a@b.com",
            "--subject",
            "x",
            "--body",
            "-",
        ]);
        match cli.command {
            Command::Messages {
                cmd: MessagesCmd::Send(a),
            } => {
                assert_eq!(a.to, vec!["a@b.com"]);
                assert_eq!(a.subject, "x");
                assert_eq!(a.body, "-");
                assert!(!a.html);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn send_multiple_recipients_and_attachments() {
        let cli = parse(&[
            "protonmail-cli",
            "messages",
            "send",
            "--to",
            "a@b.com",
            "--to",
            "c@d.com",
            "--cc",
            "e@f.com",
            "--subject",
            "hi",
            "--body",
            "hello",
            "--html",
            "--attach",
            "/tmp/x.pdf",
            "--attach",
            "/tmp/y.png",
        ]);
        match cli.command {
            Command::Messages {
                cmd: MessagesCmd::Send(a),
            } => {
                assert_eq!(a.to.len(), 2);
                assert_eq!(a.cc, vec!["e@f.com"]);
                assert!(a.html);
                assert_eq!(a.attach.len(), 2);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn read_with_html_format() {
        let cli = parse(&[
            "protonmail-cli",
            "messages",
            "read",
            "REF",
            "--format",
            "html",
        ]);
        match cli.command {
            Command::Messages {
                cmd:
                    MessagesCmd::Read {
                        reference,
                        format,
                        body_only,
                        ..
                    },
            } => {
                assert_eq!(reference, "REF");
                assert!(matches!(format, ReadFormat::Html));
                assert!(!body_only);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn read_defaults_to_text() {
        let cli = parse(&["protonmail-cli", "messages", "read", "abc"]);
        match cli.command {
            Command::Messages {
                cmd: MessagesCmd::Read { format, .. },
            } => {
                assert!(matches!(format, ReadFormat::Text));
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn move_requires_dest() {
        let cli = parse(&[
            "protonmail-cli",
            "messages",
            "move",
            "REF",
            "--dest",
            "archive",
        ]);
        match cli.command {
            Command::Messages {
                cmd: MessagesCmd::Move { references, dest },
            } => {
                assert_eq!(references, vec!["REF"]);
                assert_eq!(dest, "archive");
            }
            _ => panic!("wrong command"),
        }
        // Missing --dest is a parse error.
        assert!(Cli::try_parse_from(["protonmail-cli", "messages", "move", "REF"]).is_err());
    }

    #[test]
    fn mark_state_then_refs() {
        let cli = parse(&["protonmail-cli", "messages", "mark", "read", "id1", "id2"]);
        match cli.command {
            Command::Messages {
                cmd: MessagesCmd::Mark { state, references },
            } => {
                assert!(matches!(state, ReadState::Read));
                assert!(state.as_bool());
                assert_eq!(references, vec!["id1", "id2"]);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn global_flags_parse() {
        let cli = parse(&[
            "protonmail-cli",
            "--json",
            "--profile",
            "work",
            "messages",
            "list",
            "--folder",
            "archive",
            "--page-size",
            "10",
            "--unread",
        ]);
        assert!(cli.json);
        assert_eq!(cli.profile, "work");
        match cli.command {
            Command::Messages {
                cmd:
                    MessagesCmd::List {
                        folder,
                        page_size,
                        unread,
                        ..
                    },
            } => {
                assert_eq!(folder, "archive");
                assert_eq!(page_size, 10);
                assert!(unread);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn conversations_mark_default_folder() {
        let cli = parse(&["protonmail-cli", "conversations", "mark", "unread", "conv1"]);
        match cli.command {
            Command::Conversations {
                cmd: ConversationsCmd::Mark { state, folder, ids },
            } => {
                assert!(matches!(state, ReadState::Unread));
                assert_eq!(folder, "all");
                assert_eq!(ids, vec!["conv1"]);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn labels_create_defaults() {
        let cli = parse(&["protonmail-cli", "labels", "create", "--name", "Work"]);
        match cli.command {
            Command::Labels {
                cmd:
                    super::cli::LabelsCmd::Create {
                        name,
                        color,
                        folder,
                        parent,
                    },
            } => {
                assert_eq!(name, "Work");
                assert_eq!(color, "#8080FF");
                assert!(!folder);
                assert!(parent.is_none());
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn render_time_formats() {
        // 2021-01-01T00:00:00Z
        assert_eq!(
            super::render::fmt_time(1_609_459_200),
            "2021-01-01 00:00:00 UTC"
        );
        assert_eq!(super::render::fmt_time(0), "-");
    }
}
