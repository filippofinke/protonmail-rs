//! Contacts API (`/contacts/v4/contacts`) — read access.

use crate::error::Result;
use crate::transport::{Doer, Request};
use serde::{Deserialize, Serialize};

/// A contact email entry (plaintext metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContactEmail {
    /// Contact-email entry ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Email address.
    #[serde(default)]
    pub email: String,
    /// Display name for this email entry.
    #[serde(default)]
    pub name: String,
    /// ID of the contact this email belongs to.
    #[serde(rename = "ContactID", default)]
    pub contact_id: String,
}

/// A contact (name + its email entries).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Contact {
    /// Contact ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Contact display name.
    #[serde(default)]
    pub name: String,
    /// Email entries belonging to this contact.
    #[serde(rename = "ContactEmails", default)]
    pub emails: Vec<ContactEmail>,
}

#[derive(Deserialize)]
struct ContactsResp {
    #[serde(rename = "Contacts", default)]
    contacts: Vec<Contact>,
    #[serde(rename = "Total", default)]
    total: u32,
}

#[derive(Deserialize)]
struct EmailsResp {
    #[serde(rename = "ContactEmails", default)]
    contact_emails: Vec<ContactEmail>,
}

/// List contacts (paged).
pub async fn list<D: Doer>(d: &D, page: u32, page_size: u32) -> Result<(u32, Vec<Contact>)> {
    let req = Request::get("/contacts/v4/contacts")
        .query("Page", page.to_string())
        .query("PageSize", page_size.to_string());
    let r: ContactsResp = d.decode(req).await?;
    Ok((r.total, r.contacts))
}

/// List contact email entries, optionally filtered by an email address.
pub async fn list_emails<D: Doer>(
    d: &D,
    page: u32,
    page_size: u32,
    email: Option<&str>,
) -> Result<Vec<ContactEmail>> {
    let mut req = Request::get("/contacts/v4/contacts/emails")
        .query("Page", page.to_string())
        .query("PageSize", page_size.to_string());
    if let Some(e) = email {
        req = req.query("Email", e);
    }
    let r: EmailsResp = d.decode(req).await?;
    Ok(r.contact_emails)
}
