//! Sending: draft → upload → package → send, plus reply / forward.

use super::Client;
use crate::api;
use crate::crypto::{self, keys::AddressKeys, SessionKeyMaterial};
use crate::error::{Error, Result};
use crate::model::enums::package_type;
use base64::{engine::general_purpose::STANDARD, Engine};
use proton_srp::{SRPAuth, SRPVerifierB64};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

/// Options for composing a message.
#[derive(Debug, Default, Clone)]
pub struct SendOptions {
    /// Primary (To) recipient addresses.
    pub to: Vec<String>,
    /// Carbon-copy (Cc) recipient addresses.
    pub cc: Vec<String>,
    /// Blind carbon-copy (Bcc) recipient addresses.
    pub bcc: Vec<String>,
    /// Sender address to send from (defaults to the primary address).
    pub from: Option<String>,
    /// Message subject.
    pub subject: String,
    /// Message body content.
    pub body: String,
    /// Whether the body is HTML (otherwise plain text).
    pub html: bool,
    /// File paths to attach.
    pub attachments: Vec<PathBuf>,
    /// Unix delivery time for scheduled send.
    pub send_at: Option<i64>,
    /// Self-destruct lifetime in seconds.
    pub expires_in: Option<i64>,
}

fn dedupe(lists: &[&[String]]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for list in lists {
        for e in *list {
            let key = e.to_ascii_lowercase();
            if seen.insert(key) {
                out.push(e.clone());
            }
        }
    }
    out
}

pub(crate) fn recipient_list(addrs: &[String]) -> Value {
    Value::Array(
        addrs
            .iter()
            .map(|a| json!({ "Address": a, "Name": a }))
            .collect(),
    )
}

fn guess_mime(path: &std::path::Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("pdf") => "application/pdf",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("txt") => "text/plain",
        Some("html") => "text/html",
        Some("zip") => "application/zip",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Carried between draft-creation and packaging.
struct Uploaded {
    id: String,
    material: SessionKeyMaterial,
}

impl Client {
    /// Send a freshly-composed message.
    pub async fn send(&self, opts: &SendOptions) -> Result<String> {
        self.send_with_parent(opts, None, None, &(String::new(), Vec::new()), None)
            .await
    }

    /// Send with encrypted-outside (password-protected) for keyless external
    /// recipients. Internal/PGP recipients still get E2E.
    pub async fn send_eo(
        &self,
        opts: &SendOptions,
        password: &str,
        hint: Option<&str>,
    ) -> Result<String> {
        let eo = (password.to_string(), hint.unwrap_or("").to_string());
        self.send_with_parent(opts, None, None, &(String::new(), Vec::new()), Some(&eo))
            .await
    }

    /// Reply to a message.
    pub async fn reply(&self, reference: &str, all: bool, opts: &SendOptions) -> Result<String> {
        let parent_id = self.resolve_ref(reference).await?;
        let parent = api::messages::get_message(self.http(), &parent_id).await?;
        let quoted = self.read_message(&parent_id).await.ok();

        let mut o = opts.clone();
        // Recipients: reply → ReplyTos (fallback sender); reply-all adds To+CC.
        let reply_to: Vec<String> = if !parent.reply_tos.is_empty() {
            parent.reply_tos.iter().map(|r| r.address.clone()).collect()
        } else {
            vec![parent.meta.sender.address.clone()]
        };
        o.to = reply_to;
        if all {
            let mut cc: Vec<String> = parent
                .meta
                .to_list
                .iter()
                .chain(parent.meta.cc_list.iter())
                .map(|r| r.address.clone())
                .collect();
            let own: std::collections::HashSet<String> = self
                .keys()
                .addresses
                .iter()
                .map(|a| a.email.to_ascii_lowercase())
                .collect();
            cc.retain(|e| !own.contains(&e.to_ascii_lowercase()));
            o.cc = cc;
        }
        o.subject = with_prefix(&parent.meta.subject, "Re: ");
        if let Some(q) = quoted {
            o.body = format!("{}\n\n{}", o.body, quote_block(&parent, &q.body, o.html));
        }
        self.send_with_parent(
            &o,
            Some(&parent_id),
            Some(0),
            &(String::new(), Vec::new()),
            None,
        )
        .await
    }

    /// Forward a message (carries the original's attachments).
    pub async fn forward(&self, reference: &str, opts: &SendOptions) -> Result<String> {
        let parent_id = self.resolve_ref(reference).await?;
        let parent = api::messages::get_message(self.http(), &parent_id).await?;
        let quoted = self.read_message(&parent_id).await.ok();

        let mut o = opts.clone();
        o.subject = with_prefix(&parent.meta.subject, "Fw: ");
        if let Some(q) = quoted {
            o.body = format!("{}\n\n{}", o.body, quote_block(&parent, &q.body, o.html));
        }
        let inherited: Vec<(String, String)> = parent
            .attachments
            .iter()
            .filter_map(|a| a.key_packets.clone().map(|kp| (a.id.clone(), kp)))
            .collect();
        let src_addr_id = parent.meta.address_id.clone();
        self.send_with_parent(
            &o,
            Some(&parent_id),
            Some(2),
            &(src_addr_id, inherited),
            None,
        )
        .await
    }

    /// Cancel a scheduled send.
    pub async fn cancel_send(&self, reference: &str) -> Result<()> {
        let id = self.resolve_ref(reference).await?;
        api::messages::cancel_send(self.http(), &id).await
    }

    async fn send_with_parent(
        &self,
        opts: &SendOptions,
        parent_id: Option<&str>,
        action: Option<u8>,
        forwarded: &(String, Vec<(String, String)>),
        eo: Option<&(String, String)>,
    ) -> Result<String> {
        let provider = crypto::provider();
        let addr = match &opts.from {
            Some(e) => self.keys().address_for_email(e),
            None => self.keys().primary_address(),
        }
        .ok_or_else(|| Error::NotFound {
            kind: "sender address".into(),
        })?;
        let mime_type = if opts.html { "text/html" } else { "text/plain" };
        tracing::info!(target: "proton_core::send", from = %addr.email, to = opts.to.len(), cc = opts.cc.len(), bcc = opts.bcc.len(), attachments = opts.attachments.len(), mime = mime_type, "send: starting pipeline");

        // 1. create draft (self-encrypted body).
        tracing::debug!(target: "proton_core::send", "send step 1: creating encrypted draft (POST /mail/v4/messages)");
        let armored = crypto::encrypt_self_draft(&provider, addr, &opts.body)?;
        let mut draft = json!({
            "Message": {
                "ToList": recipient_list(&opts.to),
                "CCList": recipient_list(&opts.cc),
                "BCCList": recipient_list(&opts.bcc),
                "Subject": opts.subject,
                "Sender": { "Address": addr.email, "Name": "" },
                "Body": armored,
                "MIMEType": mime_type,
            }
        });
        if let Some(p) = parent_id {
            draft["ParentID"] = json!(p);
        }
        if let Some(a) = action {
            draft["Action"] = json!(a);
        }
        // Forwarded attachments: re-key parent attachment session keys to us.
        if !forwarded.1.is_empty() {
            let (src_addr_id, atts) = forwarded;
            let src = self.keys().address(src_addr_id).unwrap_or(addr);
            let mut akp = Map::new();
            for (att_id, kp_b64) in atts {
                let rewrapped =
                    crypto::rewrap_attachment_session_key(&provider, src, addr, kp_b64)?;
                akp.insert(att_id.clone(), json!(STANDARD.encode(rewrapped)));
            }
            draft["AttachmentKeyPackets"] = Value::Object(akp);
        }

        let created = api::messages::create_draft(self.http(), draft).await?;
        let message_id = created.meta.id.clone();
        tracing::debug!(target: "proton_core::send", message_id = %message_id, "send: draft created");

        // Run the rest; on any failure delete the draft.
        match self
            .finish_send(&provider, addr, &message_id, mime_type, opts, eo)
            .await
        {
            Ok(()) => {
                tracing::info!(target: "proton_core::send", message_id = %message_id, "send: complete");
                Ok(message_id)
            }
            Err(e) => {
                tracing::warn!(target: "proton_core::send", message_id = %message_id, error = %e, "send failed — deleting draft");
                let _ = api::messages::delete(self.http(), &[message_id]).await;
                Err(e)
            }
        }
    }

    async fn finish_send<P: proton_crypto::crypto::PGPProviderSync>(
        &self,
        provider: &P,
        addr: &AddressKeys,
        message_id: &str,
        mime_type: &str,
        opts: &SendOptions,
        eo: Option<&(String, String)>,
    ) -> Result<()> {
        // 2. upload attachments.
        let mut uploaded: Vec<Uploaded> = Vec::new();
        for path in &opts.attachments {
            let bytes = std::fs::read(path)?;
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("attachment")
                .to_string();
            let mime = guess_mime(path);
            let up = crypto::encrypt_attachment(provider, addr, &bytes)?;
            let att_id = api::attachments::upload_attachment(
                self.http(),
                &name,
                message_id,
                "",
                &mime,
                &up.key_packet,
                &up.data_packet,
                &up.signature,
            )
            .await?;
            uploaded.push(Uploaded {
                id: att_id,
                material: up.material,
            });
        }

        if !uploaded.is_empty() {
            tracing::debug!(target: "proton_core::send", count = uploaded.len(), "send step 2: attachments uploaded");
        }

        // 3. encrypt the transport body.
        let (material, data_packet) = crypto::encrypt_for_transport(provider, addr, &opts.body)?;
        let algo = crypto::algo_name(material.algo);

        // 4. per-recipient packaging.
        let recipients = dedupe(&[&opts.to, &opts.cc, &opts.bcc]);
        tracing::debug!(target: "proton_core::send", recipients = recipients.len(), "send step 3: resolving recipient keys + building packages");
        let mut internal_addrs = Map::new();
        let mut pgp_addrs = Map::new();
        let mut eo_addrs = Map::new();
        let mut external_addrs = Map::new();
        let mut modulus_cache: Option<(String, String)> = None;

        // Build a per-recipient encrypted entry (Type 1 or Type 8) wrapping the
        // body + attachment session keys to `pubkey`.
        let entry_for = |pubkey: &str, pkg_type: i64| -> Result<Value> {
            let bkp = crypto::wrap_session_key(provider, pubkey, &material)?;
            let mut entry = json!({
                "Type": pkg_type,
                "BodyKeyPacket": STANDARD.encode(&bkp),
                "Signature": 0,
            });
            if !uploaded.is_empty() {
                let mut akp = Map::new();
                for u in &uploaded {
                    akp.insert(
                        u.id.clone(),
                        json!(STANDARD.encode(crypto::wrap_session_key(
                            provider,
                            pubkey,
                            &u.material
                        )?)),
                    );
                }
                entry["AttachmentKeyPackets"] = Value::Object(akp);
            }
            Ok(entry)
        };

        for email in &recipients {
            let rk = api::keys::get_all_public_keys(self.http(), email).await?;
            // Internal = Proton-verified keys present → Type 1 (E2E).
            // External with a published PGP/WKD key → Type 8 (PGP-inline).
            // Otherwise → Type 4 (cleartext, with a warning).
            if let Some(key) = api::keys::pick_encryption_key(&rk.keys) {
                tracing::debug!(target: "proton_core::send", recipient = %email, "internal recipient — E2E package (Type 1)");
                internal_addrs.insert(email.clone(), entry_for(&key.public_key, package_type::PM)?);
            } else if let Some(key) = api::keys::pick_encryption_key(&rk.unverified) {
                tracing::debug!(target: "proton_core::send", recipient = %email, "external recipient with PGP key — PGP-inline (Type 8)");
                pgp_addrs.insert(
                    email.clone(),
                    entry_for(&key.public_key, package_type::PGP_INLINE)?,
                );
            } else if let Some((password, hint)) = eo {
                // Encrypted-outside (password-protected) recipient — Type 2.
                let (modulus, modulus_id) = match &modulus_cache {
                    Some(m) => m.clone(),
                    None => {
                        let m = api::keys::get_modulus(self.http()).await?;
                        modulus_cache = Some(m.clone());
                        m
                    }
                };
                let verifier = SRPAuth::generate_verifier_with_pgp(password, None, &modulus)
                    .map_err(|e| Error::Srp(format!("EO verifier: {e}")))?;
                let v: SRPVerifierB64 = verifier.into();
                let (_fresh, token) = crypto::new_session_key(provider)?;
                let enc_token = crypto::encrypt_text_with_password(provider, password, &token)?;
                let body_kp =
                    crypto::wrap_session_key_with_password(provider, password, &material)?;
                let mut entry = json!({
                    "Type": package_type::EO,
                    "Signature": 0,
                    "Token": token,
                    "EncToken": enc_token,
                    "BodyKeyPacket": STANDARD.encode(&body_kp),
                    "Auth": {
                        "Version": u8::from(v.version),
                        "ModulusID": modulus_id,
                        "Salt": v.salt,
                        "Verifier": v.verifier,
                    },
                });
                if !hint.is_empty() {
                    entry["PasswordHint"] = json!(hint);
                }
                if !uploaded.is_empty() {
                    let mut akp = Map::new();
                    for u in &uploaded {
                        akp.insert(
                            u.id.clone(),
                            json!(STANDARD.encode(crypto::wrap_session_key_with_password(
                                provider,
                                password,
                                &u.material
                            )?)),
                        );
                    }
                    entry["AttachmentKeyPackets"] = Value::Object(akp);
                }
                tracing::debug!(target: "proton_core::send", recipient = %email, "external recipient — encrypted-outside (Type 2)");
                eo_addrs.insert(email.clone(), entry);
            } else {
                tracing::warn!(target: "proton_core::send", recipient = %email, "external recipient with no key — sending CLEARTEXT (Type 4)");
                external_addrs.insert(
                    email.clone(),
                    json!({ "Type": package_type::CLEAR, "Signature": 0 }),
                );
            }
        }

        // 5. build packages.
        let mut packages: Vec<Value> = Vec::new();
        if !internal_addrs.is_empty() {
            let body_kp = crypto::wrap_session_key_to_self(provider, addr, &material)?;
            packages.push(json!({
                "Addresses": Value::Object(internal_addrs),
                "MIMEType": mime_type,
                "Type": package_type::PM,
                "Body": STANDARD.encode(&data_packet),
                "BodyKeyPacket": STANDARD.encode(&body_kp),
            }));
        }
        if !pgp_addrs.is_empty() {
            // PGP-inline is plaintext; the encrypted body is reused.
            let body_kp = crypto::wrap_session_key_to_self(provider, addr, &material)?;
            packages.push(json!({
                "Addresses": Value::Object(pgp_addrs),
                "MIMEType": "text/plain",
                "Type": package_type::PGP_INLINE,
                "Body": STANDARD.encode(&data_packet),
                "BodyKeyPacket": STANDARD.encode(&body_kp),
            }));
        }
        if !eo_addrs.is_empty() {
            packages.push(json!({
                "Addresses": Value::Object(eo_addrs),
                "MIMEType": mime_type,
                "Type": package_type::EO,
                "Body": STANDARD.encode(&data_packet),
            }));
        }
        if !external_addrs.is_empty() {
            let mut pkg = json!({
                "Addresses": Value::Object(external_addrs),
                "MIMEType": mime_type,
                "Type": package_type::CLEAR,
                "Body": STANDARD.encode(&data_packet),
                "BodyKey": { "Key": STANDARD.encode(&material.key), "Algorithm": algo },
            });
            if !uploaded.is_empty() {
                let mut ak = Map::new();
                for u in &uploaded {
                    ak.insert(
                        u.id.clone(),
                        json!({ "Key": STANDARD.encode(&u.material.key), "Algorithm": crypto::algo_name(u.material.algo) }),
                    );
                }
                pkg["AttachmentKeys"] = Value::Object(ak);
            }
            packages.push(pkg);
        }

        // 6. send.
        let mut body = json!({
            "ExpirationTime": Value::Null,
            "AutoSaveContacts": 0,
            "Packages": packages,
        });
        if let Some(t) = opts.send_at {
            body["DeliveryTime"] = json!(t);
        }
        if let Some(s) = opts.expires_in {
            body["ExpiresIn"] = json!(s);
        }
        tracing::debug!(target: "proton_core::send", packages = packages.len(), "send step 4: submitting (POST /mail/v4/messages/{{id}})");
        api::messages::send_message(self.http(), message_id, body).await
    }
}

fn with_prefix(subject: &str, prefix: &str) -> String {
    let p = prefix.trim_end().trim_end_matches(':').to_ascii_lowercase();
    if subject.to_ascii_lowercase().starts_with(&p) {
        subject.to_string()
    } else {
        format!("{prefix}{subject}")
    }
}

fn quote_block(parent: &crate::model::message::Message, body: &str, html: bool) -> String {
    let who = if parent.meta.sender.name.is_empty() {
        parent.meta.sender.address.clone()
    } else {
        format!(
            "{} <{}>",
            parent.meta.sender.name, parent.meta.sender.address
        )
    };
    if html {
        format!(
            "<div class=\"protonmail_quote\">On {who} wrote:<blockquote class=\"protonmail_quote\" type=\"cite\">{body}</blockquote></div>"
        )
    } else {
        let quoted: String = body.lines().map(|l| format!("> {l}\n")).collect();
        format!("------- Original Message -------\nOn {who} wrote:\n{quoted}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupe_case_insensitive() {
        let a = vec!["A@x.com".to_string(), "b@x.com".to_string()];
        let b = vec!["a@x.com".to_string(), "c@x.com".to_string()];
        let out = dedupe(&[&a, &b]);
        assert_eq!(out, vec!["A@x.com", "b@x.com", "c@x.com"]);
    }

    #[test]
    fn subject_prefix() {
        assert_eq!(with_prefix("Hello", "Re: "), "Re: Hello");
        assert_eq!(with_prefix("Re: Hello", "Re: "), "Re: Hello");
        assert_eq!(with_prefix("RE: Hello", "Re: "), "RE: Hello");
    }

    #[test]
    fn mime_guess() {
        assert_eq!(guess_mime(std::path::Path::new("a.pdf")), "application/pdf");
        assert_eq!(
            guess_mime(std::path::Path::new("a.unknownext")),
            "application/octet-stream"
        );
    }
}
