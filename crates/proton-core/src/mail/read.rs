//! Reading: list / search / read messages and conversations (with decryption).

use super::Client;
use crate::api::{self, messages::ListQuery};
use crate::crypto::{self, Verdict};
use crate::error::{Error, Result};
use crate::model::enums::resolve_folder;
use crate::model::message::{Attachment, Message, MessageMetadata};
use crate::model::Conversation;

/// A decrypted message ready for display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FullMessage {
    /// Message metadata (headers, flags, labels).
    pub meta: MessageMetadata,
    /// Decrypted body content.
    pub body: String,
    /// MIME type of the body (`text/html` or `text/plain`).
    pub mime_type: String,
    /// Signature-verification verdict for the body.
    pub verdict: Verdict,
    /// The message's attachments.
    pub attachments: Vec<Attachment>,
}

/// Search options.
#[derive(Debug, Default, Clone)]
pub struct SearchOpts {
    /// Free-text keyword to match.
    pub keyword: Option<String>,
    /// Filter by sender address.
    pub from: Option<String>,
    /// Filter by recipient address.
    pub to: Option<String>,
    /// Filter by subject text.
    pub subject: Option<String>,
    /// Only messages on or after this date (`YYYY-MM-DD`).
    pub after: Option<String>,
    /// Only messages on or before this date (`YYYY-MM-DD`).
    pub before: Option<String>,
    /// Restrict to this folder (defaults to all mail).
    pub folder: Option<String>,
    /// Only unread messages.
    pub unread: bool,
    /// Maximum number of results to return.
    pub limit: Option<u32>,
}

fn parse_date(s: &str) -> Option<i64> {
    let d = chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()?;
    Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp())
}

fn looks_like_id(r: &str) -> bool {
    r.len() >= 60 && r.ends_with("==") && !r.contains(' ')
}

/// Extract the display part from a decrypted MIME body.
fn extract_mime(raw: &str) -> (String, String) {
    let parsed = mail_parser::MessageParser::default().parse(raw.as_bytes());
    if let Some(msg) = parsed {
        if let Some(html) = msg.body_html(0) {
            return (html.into_owned(), "text/html".to_string());
        }
        if let Some(text) = msg.body_text(0) {
            return (text.into_owned(), "text/plain".to_string());
        }
    }
    (raw.to_string(), "text/plain".to_string())
}

impl Client {
    /// List a folder's messages (newest first), returning the total count and the page.
    pub async fn list_messages(
        &self,
        folder: &str,
        page: u32,
        page_size: u32,
        unread: bool,
    ) -> Result<(u32, Vec<MessageMetadata>)> {
        let q = ListQuery {
            label_id: Some(resolve_folder(folder)),
            page: Some(page),
            page_size: Some(page_size),
            unread,
            ..Default::default()
        };
        api::messages::list_messages(self.http(), &q).await
    }

    /// List a folder's conversations (newest first), returning the total count and the page.
    pub async fn list_conversations(
        &self,
        folder: &str,
        page: u32,
        page_size: u32,
        unread: bool,
    ) -> Result<(u32, Vec<Conversation>)> {
        let q = ListQuery {
            label_id: Some(resolve_folder(folder)),
            page: Some(page),
            page_size: Some(page_size),
            unread,
            ..Default::default()
        };
        api::conversations::list_conversations(self.http(), &q).await
    }

    fn search_query(&self, opts: &SearchOpts) -> ListQuery {
        ListQuery {
            label_id: Some(resolve_folder(opts.folder.as_deref().unwrap_or("all"))),
            page_size: Some(opts.limit.unwrap_or(25)),
            unread: opts.unread,
            keyword: opts.keyword.clone(),
            from: opts.from.clone(),
            to: opts.to.clone(),
            subject: opts.subject.clone(),
            begin: opts.after.as_deref().and_then(parse_date),
            end: opts.before.as_deref().and_then(parse_date),
            ..Default::default()
        }
    }

    /// Search messages server-side using the given criteria.
    pub async fn search_messages(&self, opts: &SearchOpts) -> Result<Vec<MessageMetadata>> {
        let q = self.search_query(opts);
        Ok(api::messages::list_messages(self.http(), &q).await?.1)
    }

    /// Search conversations server-side using the given criteria.
    pub async fn search_conversations(&self, opts: &SearchOpts) -> Result<Vec<Conversation>> {
        let q = self.search_query(opts);
        Ok(api::conversations::list_conversations(self.http(), &q)
            .await?
            .1)
    }

    /// Resolve a free-text reference to a message ID (exact ID, or unique search hit).
    pub async fn resolve_ref(&self, r: &str) -> Result<String> {
        if looks_like_id(r) {
            return Ok(r.to_string());
        }
        let hits = self
            .search_messages(&SearchOpts {
                keyword: Some(r.to_string()),
                folder: Some("all".into()),
                limit: Some(20),
                ..Default::default()
            })
            .await?;
        match hits.len() {
            0 => Err(Error::NotFound {
                kind: "message".into(),
            }),
            1 => Ok(hits[0].id.clone()),
            n => Err(Error::Ambiguous(n)),
        }
    }

    async fn decrypt_message(&self, m: &Message) -> Result<FullMessage> {
        let provider = crypto::provider();
        let addr = self
            .keys()
            .address(&m.meta.address_id)
            .or_else(|| self.keys().primary_address())
            .ok_or_else(|| Error::Crypto("no address key for message".into()))?;
        let sender_pubs = self.sender_pubkeys(&m.meta.sender.address).await;
        let (body, verdict) = crypto::decrypt_body(&provider, addr, &sender_pubs, &m.body)?;
        let (body, mime_type) = if m.mime_type.starts_with("multipart/") {
            extract_mime(&body)
        } else {
            (
                body,
                if m.mime_type.is_empty() {
                    "text/plain".into()
                } else {
                    m.mime_type.clone()
                },
            )
        };
        // Sanitize HTML bodies (strip scripts / active content) for safe rendering.
        let body = if mime_type == "text/html" {
            crate::html::sanitize(&body)
        } else {
            body
        };
        Ok(FullMessage {
            meta: m.meta.clone(),
            body,
            mime_type,
            verdict,
            attachments: m.attachments.clone(),
        })
    }

    /// Fetch and decrypt a single message for display.
    pub async fn read_message(&self, id: &str) -> Result<FullMessage> {
        tracing::debug!(target: "proton_core::mail", message_id = %id, "read_message: fetching + decrypting");
        let msg = api::messages::get_message(self.http(), id).await?;
        self.decrypt_message(&msg).await
    }

    /// Fetch a conversation and decrypt all of its messages (oldest first).
    pub async fn read_conversation(&self, id: &str) -> Result<(Conversation, Vec<FullMessage>)> {
        let (conv, mut msgs) = api::conversations::get_conversation(self.http(), id).await?;
        msgs.sort_by_key(|m| m.meta.time);
        let mut out = Vec::with_capacity(msgs.len());
        for m in &msgs {
            // Proton returns the full body only for the latest message; lazy-load the rest.
            let full = if m.body.is_empty() {
                api::messages::get_message(self.http(), &m.meta.id).await?
            } else {
                m.clone()
            };
            out.push(self.decrypt_message(&full).await?);
        }
        Ok((conv, out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_parsing() {
        assert!(parse_date("2024-01-15").is_some());
        assert!(parse_date("nonsense").is_none());
    }

    #[test]
    fn id_detection() {
        assert!(looks_like_id(&format!("{}==", "a".repeat(70))));
        assert!(!looks_like_id("hello world"));
        assert!(!looks_like_id("short"));
    }
}
