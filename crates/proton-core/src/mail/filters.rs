//! Sieve filter management.

use super::Client;
use crate::api;
use crate::api::filters::Filter;
use crate::error::Result;

impl Client {
    /// List the account's Sieve filters.
    pub async fn list_filters(&self) -> Result<Vec<Filter>> {
        api::filters::list(self.http()).await
    }
    /// Create a filter from a raw Sieve script.
    pub async fn create_filter(&self, name: &str, sieve: &str) -> Result<Filter> {
        api::filters::create(self.http(), name, sieve).await
    }
    /// Validate a Sieve script without creating a filter.
    pub async fn check_filter(&self, sieve: &str) -> Result<()> {
        api::filters::check(self.http(), sieve).await
    }
    /// Delete a filter by ID.
    pub async fn delete_filter(&self, id: &str) -> Result<()> {
        api::filters::delete(self.http(), id).await
    }
    /// Enable a filter.
    pub async fn enable_filter(&self, id: &str) -> Result<()> {
        api::filters::enable(self.http(), id).await
    }
    /// Disable a filter.
    pub async fn disable_filter(&self, id: &str) -> Result<()> {
        api::filters::disable(self.http(), id).await
    }
}
