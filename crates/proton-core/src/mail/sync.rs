//! Incremental sync (event stream) + local-cache reads.

use super::Client;
use crate::api;
use crate::api::events::action;
use crate::api::messages::ListQuery;
use crate::cache::Cache;
use crate::error::Result;
use crate::model::enums::resolve_folder;
use crate::model::message::MessageMetadata;

/// Outcome of a sync run.
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// Number of messages created.
    pub created: usize,
    /// Number of messages updated.
    pub updated: usize,
    /// Number of messages deleted.
    pub deleted: usize,
    /// The event cursor after applying this sync.
    pub event_id: String,
    /// First sync — the cursor was initialized (no deltas applied).
    pub initialized: bool,
}

impl Client {
    pub(crate) fn open_cache(&self) -> Result<Cache> {
        let path = Cache::default_path(&self.profile)?;
        Cache::open(&path)
    }

    /// Apply incremental events from the cached cursor into the local cache.
    pub async fn sync(&self) -> Result<SyncReport> {
        let cache = self.open_cache()?;
        let mut id = match cache.last_event_id()? {
            Some(i) => i,
            None => {
                // Bootstrap: record the current cursor; deltas flow from here.
                let latest = api::events::get_latest_event_id(self.http()).await?;
                cache.set_last_event_id(&latest)?;
                tracing::info!(target: "proton_core::sync", event_id = %latest, "sync initialized cursor");
                return Ok(SyncReport {
                    created: 0,
                    updated: 0,
                    deleted: 0,
                    event_id: latest,
                    initialized: true,
                });
            }
        };

        let (mut created, mut updated, mut deleted) = (0, 0, 0);
        loop {
            let batch = api::events::get_events(self.http(), &id).await?;
            if batch.refresh {
                tracing::warn!(target: "proton_core::sync", "server requested full resync — clearing cache");
                cache.clear()?;
            }
            for ev in &batch.messages {
                match ev.action {
                    action::DELETE => {
                        cache.delete_message(&ev.id)?;
                        deleted += 1;
                    }
                    action::CREATE | action::UPDATE | action::UPDATE_FLAGS => {
                        if let Some(m) = &ev.message {
                            cache.upsert_message(m)?;
                            if ev.action == action::CREATE {
                                created += 1;
                            } else {
                                updated += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
            cache.set_last_event_id(&batch.event_id)?;
            id = batch.event_id;
            if !batch.more {
                break;
            }
        }
        tracing::info!(target: "proton_core::sync", created, updated, deleted, event_id = %id, "sync complete");
        Ok(SyncReport {
            created,
            updated,
            deleted,
            event_id: id,
            initialized: false,
        })
    }

    /// Backfill a folder into the cache by paging the API (bounded by `max_pages`).
    pub async fn cache_folder(
        &self,
        folder: &str,
        max_pages: u32,
        page_size: u32,
    ) -> Result<usize> {
        let cache = self.open_cache()?;
        let label = resolve_folder(folder);
        let mut n = 0usize;
        for page in 0..max_pages {
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
            for m in &msgs {
                cache.upsert_message(m)?;
                n += 1;
            }
            if (msgs.len() as u32) < page_size {
                break;
            }
        }
        tracing::info!(target: "proton_core::sync", folder, cached = n, "cache_folder backfill");
        Ok(n)
    }

    /// Read messages for a folder from the local cache (offline).
    pub fn cached_messages(
        &self,
        folder: &str,
        unread: bool,
        limit: u32,
    ) -> Result<Vec<MessageMetadata>> {
        self.open_cache()?
            .list(&resolve_folder(folder), unread, limit, 0)
    }

    /// Cached (total, unread) counts for a folder.
    pub fn cached_count(&self, folder: &str) -> Result<(i64, i64)> {
        self.open_cache()?.count(&resolve_folder(folder))
    }

    /// Build the local encrypted-search index: page a folder, decrypt each
    /// message body, and index it (bounded by `max_pages`).
    pub async fn index_folder(
        &self,
        folder: &str,
        max_pages: u32,
        page_size: u32,
    ) -> Result<usize> {
        let cache = self.open_cache()?;
        let label = resolve_folder(folder);
        let mut n = 0usize;
        for page in 0..max_pages {
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
                match self.read_message(&meta.id).await {
                    Ok(full) => {
                        cache.index_message(&full.meta, &full.body)?;
                        n += 1;
                    }
                    Err(_) => {
                        // Index metadata even if the body can't be decrypted.
                        let _ = cache.upsert_message(meta);
                    }
                }
            }
            if (msgs.len() as u32) < page_size {
                break;
            }
        }
        tracing::info!(target: "proton_core::sync", folder, indexed = n, "encrypted-search index built");
        Ok(n)
    }

    /// Full-text search the local index (decrypted bodies; private + offline).
    pub fn search_local(&self, query: &str, limit: u32) -> Result<Vec<MessageMetadata>> {
        self.open_cache()?.search(query, limit)
    }
}
