//! proton-mcp — an MCP server exposing Proton Mail tools backed by `proton-core`.
//!
//! Authentication is delegated to `protonmail-cli`: a human runs `protonmail-cli login`
//! first, and this server simply resumes the stored session via
//! [`proton_core::Client::resume`]. Default transport is stdio; pass `--http`
//! to serve over Streamable HTTP instead.

mod common;
mod server;
mod tools;

use anyhow::Result;
use clap::Parser;
use rmcp::transport::stdio;
use rmcp::ServiceExt;

use server::ProtonMail;

#[derive(Parser, Debug)]
#[command(
    name = "proton-mcp",
    version,
    about = "Proton Mail MCP server (backed by proton-core; reuses a `protonmail-cli login` session)"
)]
struct Args {
    /// Session profile to resume (created by `protonmail-cli login`).
    #[arg(long, default_value = "default", env = "PROTON_PROFILE")]
    profile: String,

    /// Allow destructive/structural write actions (send, reply, forward, move,
    /// trash, delete, empty-folder, drafts, filters, labels, settings, ...)
    /// without a per-call `confirm: true`. Without this flag those tools return
    /// a dry-run preview unless the call passes `confirm: true`. Benign toggles
    /// (mark, star, snooze) and reads always execute.
    #[arg(long, default_value_t = false)]
    allow_writes: bool,

    /// Serve over Streamable HTTP at this address (e.g. `127.0.0.1:8080`),
    /// mounted at `/mcp`. When omitted, the server speaks MCP over stdio.
    #[arg(long, value_name = "ADDR")]
    http: Option<String>,

    /// Verbose logging to stderr (-v debug, -vv trace core, -vvv trace all).
    /// `RUST_LOG` overrides. Logs login, crypto, and every HTTP call.
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Logs MUST go to stderr: stdout is the MCP protocol channel on stdio.
    // Default (no -v) is INFO; -v debug, -vv trace core, -vvv trace all.
    proton_core::init_tracing(args.verbose.max(1));

    let server = ProtonMail::new(args.profile.clone(), args.allow_writes);

    match args.http {
        Some(addr) => {
            tracing::info!(
                profile = %args.profile,
                allow_writes = args.allow_writes,
                %addr,
                "starting proton-mcp on Streamable HTTP (mounted at /mcp)"
            );
            server::serve_http(server, &addr).await?;
        }
        None => {
            tracing::info!(
                profile = %args.profile,
                allow_writes = args.allow_writes,
                "starting proton-mcp on stdio"
            );
            let running = server.serve(stdio()).await?;
            running.waiting().await?;
        }
    }

    Ok(())
}
