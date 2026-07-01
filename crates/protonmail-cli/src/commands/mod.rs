//! Command handlers, grouped by area.

pub mod attachments;
pub mod auth;
pub mod contacts;
pub mod conversations;
pub mod drafts;
pub mod export;
pub mod filters;
pub mod labels;
pub mod messages;
pub mod search;
pub mod sync;
pub mod watch;

use proton_core::{Client, Result};
use std::io::{Read, Write};

/// Resume a saved session for the given profile.
pub async fn resume(profile: &str) -> Result<Client> {
    Client::resume(profile).await
}

/// Resolve a body argument: `-` reads stdin, otherwise the literal text.
pub fn read_body(body: &str) -> std::io::Result<String> {
    if body == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        Ok(body.to_string())
    }
}

/// Prompt on stderr (keeping stdout clean) and read a trimmed line.
pub fn prompt_line(prompt: &str) -> std::io::Result<String> {
    eprint!("{prompt}");
    std::io::stderr().flush()?;
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

/// Resolve every free-text reference to a concrete message ID.
pub async fn resolve_all(client: &Client, refs: &[String]) -> Result<Vec<String>> {
    let mut out = Vec::with_capacity(refs.len());
    for r in refs {
        out.push(client.resolve_ref(r).await?);
    }
    Ok(out)
}
