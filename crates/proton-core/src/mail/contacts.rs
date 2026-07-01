//! Contacts (read) + address listing.

use super::Client;
use crate::api;
use crate::api::contacts::{Contact, ContactEmail};
use crate::error::Result;

/// A sending address.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AddressInfo {
    /// The address identifier.
    pub id: String,
    /// The address email.
    pub email: String,
}

impl Client {
    /// List contacts (paged), returning the total count and the page.
    pub async fn list_contacts(&self, page: u32, page_size: u32) -> Result<(u32, Vec<Contact>)> {
        api::contacts::list(self.http(), page, page_size).await
    }

    /// List contact email entries, optionally filtered by an email address.
    pub async fn list_contact_emails(&self, email: Option<&str>) -> Result<Vec<ContactEmail>> {
        api::contacts::list_emails(self.http(), 0, 100, email).await
    }

    /// The account's sending addresses (from the unlocked key store).
    pub fn addresses(&self) -> Vec<AddressInfo> {
        self.keys()
            .addresses
            .iter()
            .map(|a| AddressInfo {
                id: a.address_id.clone(),
                email: a.email.clone(),
            })
            .collect()
    }
}
