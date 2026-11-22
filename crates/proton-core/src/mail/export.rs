//! Export messages to `.eml` files (decrypt → RFC 822, attachments included).

use super::read::SearchOpts;
use super::Client;
use crate::api;
use crate::error::{Error, Result};
use crate::model::message::{Message, Recipient};
use futures::stream::{self, StreamExt};
use mail_builder::headers::address::Address;
use mail_builder::headers::date::Date;
use mail_builder::headers::message_id::MessageId;
use mail_builder::headers::text::Text;
use mail_builder::headers::{Header, HeaderType};
use mail_builder::MessageBuilder;
use std::path::Path;

/// Options controlling an export run.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Number of messages fetched + decrypted concurrently.
    pub concurrency: usize,
    /// Skip messages whose `.eml` file already exists (resume a partial run).
    pub skip_existing: bool,
    /// Download and embed attachments (otherwise only headers + body).
    pub attachments: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            concurrency: 4,
            skip_existing: false,
            attachments: true,
        }
    }
}

/// Outcome of an export run.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ExportReport {
    /// Messages written to disk.
    pub exported: usize,
    /// Messages skipped because their file already existed.
    pub skipped: usize,
    /// Messages that failed, as `(message id, error)`.
    pub failed: Vec<(String, String)>,
}

impl Client {
    /// Export up to `max` messages from a folder as `.eml` files in `out_dir`.
    /// Returns the number written.
    pub async fn export_folder(&self, folder: &str, out_dir: &Path, max: u32) -> Result<usize> {
        let opts = SearchOpts {
            folder: Some(folder.to_string()),
            limit: Some(max),
            ..Default::default()
        };
        let r = self
            .export_search(&opts, out_dir, &ExportOptions::default())
            .await?;
        Ok(r.exported)
    }

    /// Export every message matching a server-side search (`opts.limit` caps
    /// the count; folder defaults to all mail).
    pub async fn export_search(
        &self,
        opts: &SearchOpts,
        out_dir: &Path,
        eo: &ExportOptions,
    ) -> Result<ExportReport> {
        let ids: Vec<String> = self
            .search_messages(opts)
            .await?
            .into_iter()
            .map(|m| m.id)
            .collect();
        self.export_messages(&ids, out_dir, eo).await
    }

    /// Export the given message IDs as `<id>.eml` files in `out_dir`.
    ///
    /// A failure on one message is recorded in the report and does not abort
    /// the run.
    pub async fn export_messages(
        &self,
        ids: &[String],
        out_dir: &Path,
        eo: &ExportOptions,
    ) -> Result<ExportReport> {
        std::fs::create_dir_all(out_dir)?;
        let mut report = ExportReport::default();
        let mut todo = Vec::with_capacity(ids.len());
        for id in ids {
            let path = out_dir.join(format!("{}.eml", sanitize_filename(id)));
            if eo.skip_existing && path.exists() {
                report.skipped += 1;
            } else {
                todo.push((id.clone(), path));
            }
        }
        let results: Vec<(String, Result<()>)> = stream::iter(todo)
            .map(|(id, path)| async move {
                let r = async {
                    let eml = self.message_eml(&id, eo.attachments).await?;
                    // Write atomically so an interrupted run never leaves a
                    // truncated file that `skip_existing` would then trust.
                    let tmp = path.with_extension("eml.part");
                    std::fs::write(&tmp, eml)?;
                    std::fs::rename(&tmp, &path)?;
                    Ok(())
                }
                .await;
                (id, r)
            })
            .buffer_unordered(eo.concurrency.max(1))
            .collect()
            .await;
        for (id, r) in results {
            match r {
                Ok(()) => report.exported += 1,
                Err(e) => {
                    tracing::warn!(target: "proton_core::mail", message_id = %id, error = %e, "export: message failed");
                    report.failed.push((id, e.to_string()));
                }
            }
        }
        tracing::info!(
            target: "proton_core::mail",
            exported = report.exported,
            skipped = report.skipped,
            failed = report.failed.len(),
            "export_messages"
        );
        Ok(report)
    }

    /// Build the full RFC 822 representation of one message.
    pub async fn message_eml(&self, id: &str, with_attachments: bool) -> Result<Vec<u8>> {
        let msg = api::messages::get_message(self.http(), id).await?;
        let (raw, _verdict) = self.decrypt_raw_body(&msg).await?;

        // PGP/MIME: the decrypted body is already a complete MIME entity
        // (with its own Content-Type + boundary and any attachments inside).
        if msg.mime_type.starts_with("multipart/") && has_content_type_header(&raw) {
            let mut out = write_headers(&msg)?;
            out.extend_from_slice(raw.as_bytes());
            return Ok(out);
        }

        let mut b = MessageBuilder::new();
        b = if msg.mime_type.eq_ignore_ascii_case("text/html") {
            b.html_body(raw)
        } else {
            b.text_body(raw)
        };
        if with_attachments {
            for att in &msg.attachments {
                let data = self
                    .fetch_attachment(&msg, att)
                    .await
                    .map_err(|e| Error::Crypto(format!("attachment {:?}: {e}", att.name)))?;
                let ct = att
                    .mime_type
                    .clone()
                    .filter(|m| !m.is_empty())
                    .unwrap_or_else(|| "application/octet-stream".into());
                b = match (att.is_inline(), att.content_id()) {
                    (true, Some(cid)) => b.inline(ct, cid, data),
                    _ => b.attachment(ct, att.name.clone(), data),
                };
            }
        }
        let mut out = write_headers(&msg)?;
        b.write_body(&mut out)?;
        Ok(out)
    }
}

fn address(r: &Recipient) -> Address<'static> {
    if r.name.is_empty() || r.name == r.address {
        Address::new_address(None::<String>, r.address.clone())
    } else {
        Address::new_address(Some(r.name.clone()), r.address.clone())
    }
}

fn address_list(list: &[Recipient]) -> Option<HeaderType<'static>> {
    match list.len() {
        0 => None,
        1 => Some(address(&list[0]).into()),
        _ => Some(Address::new_list(list.iter().map(address).collect()).into()),
    }
}

/// Top-level RFC 822 headers (encoded per RFC 2047 where needed), ending
/// with `MIME-Version`; the body part supplies its own `Content-Type`.
fn write_headers(m: &Message) -> Result<Vec<u8>> {
    let meta = &m.meta;
    let mut headers: Vec<(&str, HeaderType)> = vec![("From", address(&meta.sender).into())];
    if let Some(h) = address_list(&meta.to_list) {
        headers.push(("To", h));
    }
    if let Some(h) = address_list(&meta.cc_list) {
        headers.push(("Cc", h));
    }
    if let Some(h) = address_list(&meta.bcc_list) {
        headers.push(("Bcc", h));
    }
    let reply_tos: Vec<Recipient> = if !m.reply_tos.is_empty() {
        m.reply_tos.clone()
    } else {
        m.reply_to.clone().into_iter().collect()
    };
    if !reply_tos.is_empty() && reply_tos.iter().any(|r| r.address != meta.sender.address) {
        if let Some(h) = address_list(&reply_tos) {
            headers.push(("Reply-To", h));
        }
    }
    headers.push(("Subject", Text::new(meta.subject.clone()).into()));
    headers.push(("Date", Date::new(meta.time).into()));
    if let Some(mid) = meta.external_id.as_deref().filter(|s| !s.is_empty()) {
        let mid = mid.trim().trim_start_matches('<').trim_end_matches('>');
        headers.push(("Message-ID", MessageId::new(mid.to_string()).into()));
    }
    let mut out = Vec::new();
    for (name, value) in &headers {
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(b": ");
        value.write_header(&mut out, name.len() + 2)?;
    }
    out.extend_from_slice(b"MIME-Version: 1.0\r\n");
    Ok(out)
}

/// True if the MIME entity's header block (before the first blank line)
/// declares a `Content-Type`.
fn has_content_type_header(raw: &str) -> bool {
    raw.lines()
        .take_while(|l| !l.trim().is_empty())
        .any(|l| l.to_ascii_lowercase().starts_with("content-type:"))
}

fn sanitize_filename(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::message::MessageMetadata;
    use mail_parser::MimeHeaders;

    fn sample() -> Message {
        let mut meta = MessageMetadata {
            subject: "Hi è".into(),
            time: 1_700_000_000,
            external_id: Some("<abc@x.com>".into()),
            ..Default::default()
        };
        meta.sender = Recipient {
            name: "Alice".into(),
            address: "a@proton.me".into(),
            ..Default::default()
        };
        meta.to_list = vec![Recipient::new("b@x.com"), Recipient::new("c@x.com")];
        Message {
            meta,
            mime_type: "text/plain".into(),
            ..Default::default()
        }
    }

    #[test]
    fn headers_are_encoded() {
        let h = String::from_utf8(write_headers(&sample()).unwrap()).unwrap();
        assert!(h.contains("From: \"Alice\" <a@proton.me>\r\n"), "{h}");
        assert!(h.contains("To: <b@x.com>, <c@x.com>\r\n"), "{h}");
        assert!(h.contains("Subject: =?utf-8?"), "{h}");
        assert!(h.contains("Message-ID: <abc@x.com>\r\n"), "{h}");
        assert!(h.ends_with("MIME-Version: 1.0\r\n"));
    }

    #[test]
    fn body_with_attachment_is_multipart_and_parses() {
        let mut out = write_headers(&sample()).unwrap();
        MessageBuilder::new()
            .text_body("body text")
            .attachment("application/pdf", "NDA.pdf", b"%PDF-1.4 x".to_vec())
            .write_body(&mut out)
            .unwrap();
        let parsed = mail_parser::MessageParser::default().parse(&out).unwrap();
        assert_eq!(parsed.subject(), Some("Hi è"));
        assert_eq!(parsed.body_text(0).as_deref(), Some("body text"));
        assert_eq!(parsed.attachment_count(), 1);
        let att = parsed.attachment(0).unwrap();
        assert_eq!(att.attachment_name(), Some("NDA.pdf"));
        assert_eq!(att.contents(), b"%PDF-1.4 x");
    }

    #[test]
    fn detects_pgp_mime_headers() {
        assert!(has_content_type_header(
            "Content-Type: multipart/mixed; boundary=\"b\"\r\n\r\n--b\r\n"
        ));
        assert!(!has_content_type_header("hello\r\n\r\nContent-Type: x"));
    }
}
