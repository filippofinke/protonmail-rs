//! Mail settings API (`/mail/v4/settings`).

use crate::error::Result;
use crate::transport::{Doer, Request};
use serde::Deserialize;

#[derive(Deserialize)]
struct Resp {
    #[serde(rename = "MailSettings", default)]
    mail_settings: serde_json::Value,
}

/// Fetch the raw mail settings object.
pub async fn get_mail_settings<D: Doer>(d: &D) -> Result<serde_json::Value> {
    let r: Resp = d.decode(Request::get("/mail/v4/settings")).await?;
    Ok(r.mail_settings)
}

async fn put_setting<D: Doer>(d: &D, path: &str, field: &str, value: i64) -> Result<()> {
    let mut body = serde_json::Map::new();
    body.insert(field.to_string(), serde_json::json!(value));
    let _: serde_json::Value = d
        .decode(
            Request::put(format!("/mail/v4/settings/{path}")).json(serde_json::Value::Object(body)),
        )
        .await?;
    Ok(())
}

/// Toggle signing outgoing mail.
pub async fn set_sign<D: Doer>(d: &D, on: bool) -> Result<()> {
    put_setting(d, "sign", "Sign", on as i64).await
}

/// Toggle attaching the public key to outgoing mail.
pub async fn set_attach_public_key<D: Doer>(d: &D, on: bool) -> Result<()> {
    put_setting(d, "attachpublic", "AttachPublicKey", on as i64).await
}
