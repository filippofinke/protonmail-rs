//! Labels / folders API (`/core/v4/labels`).

use crate::error::Result;
use crate::model::label::Label;
use crate::transport::{Doer, Request};
use serde::Deserialize;

#[derive(Deserialize)]
struct ListResp {
    #[serde(rename = "Labels", default)]
    labels: Vec<Label>,
}

#[derive(Deserialize)]
struct CreateResp {
    #[serde(rename = "Label")]
    label: Label,
}

/// List labels (`Type=1`) or folders (`Type=3`) etc.
pub async fn list<D: Doer>(d: &D, label_type: i64) -> Result<Vec<Label>> {
    let r: ListResp = d
        .decode(Request::get("/core/v4/labels").query("Type", label_type.to_string()))
        .await?;
    Ok(r.labels)
}

/// Create a label/folder. `Type` 1=label, 3=folder.
pub async fn create<D: Doer>(
    d: &D,
    name: &str,
    color: &str,
    label_type: i64,
    parent_id: Option<&str>,
) -> Result<Label> {
    let mut body = serde_json::json!({ "Name": name, "Color": color, "Type": label_type });
    if let Some(p) = parent_id {
        body["ParentID"] = serde_json::Value::String(p.to_string());
    }
    let r: CreateResp = d
        .decode(Request::post("/core/v4/labels").json(body))
        .await?;
    Ok(r.label)
}

/// Update a label/folder. `body` carries the changed fields (Name/Color/ParentID).
pub async fn update<D: Doer>(d: &D, id: &str, body: serde_json::Value) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::put(format!("/core/v4/labels/{id}")).json(body))
        .await?;
    Ok(())
}

/// Delete labels/folders by ID.
pub async fn delete<D: Doer>(d: &D, ids: &[String]) -> Result<()> {
    let body = serde_json::json!({ "LabelIDs": ids });
    let _: serde_json::Value = d
        .decode(Request::delete("/core/v4/labels").json(body))
        .await?;
    Ok(())
}
