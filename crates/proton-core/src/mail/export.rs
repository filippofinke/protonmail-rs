//! Export messages to `.eml` files (decrypt → RFC 822).

use super::read::FullMessage;
use super::Client;
use crate::api;
use crate::api::messages::ListQuery;
use crate::error::Result;
use crate::model::enums::resolve_folder;
use std::path::Path;

impl Client {
    /// Export up to `max` messages from a folder as `.eml` files in `out_dir`.
    /// Returns the number written.
    pub async fn export_folder(&self, folder: &str, out_dir: &Path, max: u32) -> Result<usize> {
        std::fs::create_dir_all(out_dir)?;
        let label = resolve_folder(folder);
        let page_size = 50u32;
        let mut written = 0u32;
        let mut page = 0u32;
        while written < max {
            let q = ListQuery {
                label_id: Some(label.clone()),
                page: Some(page),
                page_size: Some(page_size),
                ..Default::default()
            };
            let (_total, msgs) = api::messages::list_messages(self.http(), &q).await?;
            if msgs.is_empty() {
                break;
            }
            for meta in &msgs {
                if written >= max {
                    break;
                }
                let full = self.read_message(&meta.id).await?;
                let eml = build_eml(&full);
                let name = sanitize_filename(&meta.id);
                std::fs::write(out_dir.join(format!("{name}.eml")), eml)?;
                written += 1;
            }
            if (msgs.len() as u32) < page_size {
                break;
            }
            page += 1;
        }
        tracing::info!(target: "proton_core::mail", folder, exported = written, "export_folder");
        Ok(written as usize)
    }
}

fn build_eml(m: &FullMessage) -> String {
    let from = if m.meta.sender.name.is_empty() {
        m.meta.sender.address.clone()
    } else {
        format!("{} <{}>", m.meta.sender.name, m.meta.sender.address)
    };
    let to = m
        .meta
        .to_list
        .iter()
        .map(|r| r.address.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let date = chrono::DateTime::from_timestamp(m.meta.time, 0)
        .map(|d| d.to_rfc2822())
        .unwrap_or_default();
    let mime = if m.mime_type.is_empty() {
        "text/plain"
    } else {
        &m.mime_type
    };
    format!(
        "From: {from}\r\nTo: {to}\r\nSubject: {}\r\nDate: {date}\r\nMIME-Version: 1.0\r\n\
         Content-Type: {mime}; charset=utf-8\r\n\r\n{}",
        m.meta.subject, m.body
    )
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
    use crate::model::message::{MessageMetadata, Recipient};

    #[test]
    fn eml_has_headers_and_body() {
        let mut meta = MessageMetadata {
            subject: "Hi".into(),
            time: 1_700_000_000,
            ..Default::default()
        };
        meta.sender = Recipient {
            name: "Alice".into(),
            address: "a@proton.me".into(),
            ..Default::default()
        };
        meta.to_list = vec![Recipient::new("b@x.com")];
        let full = FullMessage {
            meta,
            body: "body text".into(),
            mime_type: "text/plain".into(),
            verdict: crate::crypto::Verdict::Verified,
            attachments: vec![],
        };
        let eml = build_eml(&full);
        assert!(eml.contains("From: Alice <a@proton.me>"));
        assert!(eml.contains("To: b@x.com"));
        assert!(eml.contains("Subject: Hi"));
        assert!(eml.contains("\r\n\r\nbody text"));
    }
}
