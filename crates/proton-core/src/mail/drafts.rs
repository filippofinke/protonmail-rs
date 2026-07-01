//! Draft lifecycle: save / update / list / delete (without sending).

use super::send::{recipient_list, SendOptions};
use super::Client;
use crate::api;
use crate::crypto;
use crate::error::{Error, Result};
use crate::model::message::MessageMetadata;
use serde_json::json;

impl Client {
    fn draft_message_json(&self, opts: &SendOptions) -> Result<serde_json::Value> {
        let addr = match &opts.from {
            Some(e) => self.keys().address_for_email(e),
            None => self.keys().primary_address(),
        }
        .ok_or_else(|| Error::NotFound {
            kind: "sender address".into(),
        })?;
        let provider = crypto::provider();
        let mime_type = if opts.html { "text/html" } else { "text/plain" };
        let armored = crypto::encrypt_self_draft(&provider, addr, &opts.body)?;
        Ok(json!({
            "Message": {
                "ToList": recipient_list(&opts.to),
                "CCList": recipient_list(&opts.cc),
                "BCCList": recipient_list(&opts.bcc),
                "Subject": opts.subject,
                "Sender": { "Address": addr.email, "Name": "" },
                "Body": armored,
                "MIMEType": mime_type,
            }
        }))
    }

    /// Create a draft (encrypted to self, stored in Drafts) without sending it.
    pub async fn save_draft(&self, opts: &SendOptions) -> Result<String> {
        tracing::info!(target: "proton_core::mail", to = opts.to.len(), "save_draft");
        let body = self.draft_message_json(opts)?;
        let created = api::messages::create_draft(self.http(), body).await?;
        Ok(created.meta.id)
    }

    /// Replace an existing draft's content.
    pub async fn update_draft(&self, id: &str, opts: &SendOptions) -> Result<()> {
        tracing::info!(target: "proton_core::mail", draft_id = %id, "update_draft");
        let body = self.draft_message_json(opts)?;
        api::messages::update_draft(self.http(), id, body).await?;
        Ok(())
    }

    /// List drafts (the Drafts folder).
    pub async fn list_drafts(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<(u32, Vec<MessageMetadata>)> {
        self.list_messages("drafts", page, page_size, false).await
    }

    /// Delete drafts permanently.
    pub async fn delete_draft(&self, ids: &[String]) -> Result<()> {
        api::messages::delete(self.http(), ids).await
    }
}
