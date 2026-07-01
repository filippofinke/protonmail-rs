//! Sieve filters API (`/mail/v4/filters`).

use crate::error::Result;
use crate::transport::{Doer, Request};
use serde::Deserialize;

/// A server-side (Sieve) filter.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Filter {
    /// Filter ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Filter display name.
    #[serde(default)]
    pub name: String,
    /// Status (`1` = enabled, `0` = disabled).
    #[serde(default)]
    pub status: i64,
    /// Sieve filter format version.
    #[serde(default)]
    pub version: i64,
}

#[derive(Deserialize)]
struct ListResp {
    #[serde(rename = "Filters", default)]
    filters: Vec<Filter>,
}

#[derive(Deserialize)]
struct OneResp {
    #[serde(rename = "Filter")]
    filter: Filter,
}

/// List the account's Sieve filters.
pub async fn list<D: Doer>(d: &D) -> Result<Vec<Filter>> {
    let r: ListResp = d.decode(Request::get("/mail/v4/filters")).await?;
    Ok(r.filters)
}

/// Create a filter from a raw Sieve script.
pub async fn create<D: Doer>(d: &D, name: &str, sieve: &str) -> Result<Filter> {
    let body = serde_json::json!({ "Name": name, "Sieve": sieve, "Version": 2, "Status": 1 });
    let r: OneResp = d
        .decode(Request::post("/mail/v4/filters").json(body))
        .await?;
    Ok(r.filter)
}

/// Validate a Sieve script (does not create a filter).
pub async fn check<D: Doer>(d: &D, sieve: &str) -> Result<()> {
    let body = serde_json::json!({ "Sieve": sieve, "Version": 2 });
    let _: serde_json::Value = d
        .decode(Request::put("/mail/v4/filters/check").json(body))
        .await?;
    Ok(())
}

/// Delete a filter by ID.
pub async fn delete<D: Doer>(d: &D, id: &str) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::delete(format!("/mail/v4/filters/{id}")))
        .await?;
    Ok(())
}

/// Enable a filter by ID.
pub async fn enable<D: Doer>(d: &D, id: &str) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::put(format!("/mail/v4/filters/{id}/enable")))
        .await?;
    Ok(())
}

/// Disable a filter by ID.
pub async fn disable<D: Doer>(d: &D, id: &str) -> Result<()> {
    let _: serde_json::Value = d
        .decode(Request::put(format!("/mail/v4/filters/{id}/disable")))
        .await?;
    Ok(())
}
