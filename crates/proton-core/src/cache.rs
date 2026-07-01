//! Local SQLite cache of message metadata + the sync cursor.

use crate::error::{Error, Result};
use crate::model::message::MessageMetadata;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

fn map<E: std::fmt::Display>(e: E) -> Error {
    Error::Cache(e.to_string())
}

/// A per-profile metadata cache.
pub struct Cache {
    conn: Connection,
}

impl Cache {
    /// Default cache path: `<cache_dir>/protonmail-cli/<profile>.db`.
    pub fn default_path(profile: &str) -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("me", "Proton", "protonmail-cli")
            .ok_or_else(|| Error::Cache("cannot resolve cache dir".into()))?;
        let dir = dirs.cache_dir().to_path_buf();
        std::fs::create_dir_all(&dir).map_err(map)?;
        let p = if profile.is_empty() {
            "default"
        } else {
            profile
        };
        Ok(dir.join(format!("{p}.db")))
    }

    /// Open (creating if needed) the cache database at `path`, ensuring its schema.
    pub fn open(path: &Path) -> Result<Cache> {
        let conn = Connection::open(path).map_err(map)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT);
             CREATE TABLE IF NOT EXISTS messages(id TEXT PRIMARY KEY, time INTEGER, unread INTEGER, json TEXT);
             CREATE TABLE IF NOT EXISTS message_labels(message_id TEXT, label_id TEXT, PRIMARY KEY(message_id, label_id));
             CREATE INDEX IF NOT EXISTS idx_ml_label ON message_labels(label_id);
             CREATE VIRTUAL TABLE IF NOT EXISTS msg_fts USING fts5(id UNINDEXED, subject, body, participants);",
        )
        .map_err(map)?;
        Ok(Cache { conn })
    }

    /// The stored sync cursor, or `None` if not yet initialized.
    pub fn last_event_id(&self) -> Result<Option<String>> {
        let r = self.conn.query_row(
            "SELECT value FROM meta WHERE key='last_event_id'",
            [],
            |row| row.get::<_, String>(0),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(map(e)),
        }
    }

    /// Store the sync cursor.
    pub fn set_last_event_id(&self, id: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO meta(key, value) VALUES('last_event_id', ?1)",
                [id],
            )
            .map_err(map)?;
        Ok(())
    }

    /// Insert or replace a message's cached metadata and its label memberships.
    pub fn upsert_message(&self, m: &MessageMetadata) -> Result<()> {
        let json = serde_json::to_string(m)?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO messages(id, time, unread, json) VALUES(?1, ?2, ?3, ?4)",
                rusqlite::params![m.id, m.time, m.unread, json],
            )
            .map_err(map)?;
        self.conn
            .execute("DELETE FROM message_labels WHERE message_id=?1", [&m.id])
            .map_err(map)?;
        for label in &m.label_ids {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO message_labels(message_id, label_id) VALUES(?1, ?2)",
                    rusqlite::params![m.id, label],
                )
                .map_err(map)?;
        }
        Ok(())
    }

    /// Remove a message (and its label memberships) from the cache.
    pub fn delete_message(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM messages WHERE id=?1", [id])
            .map_err(map)?;
        self.conn
            .execute("DELETE FROM message_labels WHERE message_id=?1", [id])
            .map_err(map)?;
        Ok(())
    }

    /// Cached messages in a label, newest first.
    pub fn list(
        &self,
        label_id: &str,
        unread_only: bool,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<MessageMetadata>> {
        let sql = format!(
            "SELECT m.json FROM messages m JOIN message_labels l ON m.id=l.message_id \
             WHERE l.label_id=?1 {} ORDER BY m.time DESC LIMIT ?2 OFFSET ?3",
            if unread_only { "AND m.unread=1" } else { "" }
        );
        let mut stmt = self.conn.prepare(&sql).map_err(map)?;
        let rows = stmt
            .query_map(rusqlite::params![label_id, limit, offset], |row| {
                row.get::<_, String>(0)
            })
            .map_err(map)?;
        let mut out = Vec::new();
        for r in rows {
            let json = r.map_err(map)?;
            if let Ok(m) = serde_json::from_str::<MessageMetadata>(&json) {
                out.push(m);
            }
        }
        Ok(out)
    }

    /// (total, unread) cached for a label.
    pub fn count(&self, label_id: &str) -> Result<(i64, i64)> {
        self.conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(m.unread),0) FROM messages m \
                 JOIN message_labels l ON m.id=l.message_id WHERE l.label_id=?1",
                [label_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .map_err(map)
    }

    /// Drop all cached messages (on a server-requested refresh).
    pub fn clear(&self) -> Result<()> {
        self.conn
            .execute_batch("DELETE FROM messages; DELETE FROM message_labels; DELETE FROM msg_fts;")
            .map_err(map)?;
        Ok(())
    }

    /// Index a message's decrypted body for local full-text search.
    pub fn index_message(&self, m: &MessageMetadata, body: &str) -> Result<()> {
        self.upsert_message(m)?;
        let participants = std::iter::once(m.sender.address.clone())
            .chain(m.to_list.iter().map(|r| r.address.clone()))
            .collect::<Vec<_>>()
            .join(" ");
        self.conn
            .execute("DELETE FROM msg_fts WHERE id=?1", [&m.id])
            .map_err(map)?;
        self.conn
            .execute(
                "INSERT INTO msg_fts(id, subject, body, participants) VALUES(?1, ?2, ?3, ?4)",
                rusqlite::params![m.id, m.subject, body, participants],
            )
            .map_err(map)?;
        Ok(())
    }

    /// Full-text search the local index. Tokens are AND-ed; punctuation-safe.
    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<MessageMetadata>> {
        let expr = query
            .split_whitespace()
            .map(|t| format!("\"{}\"", t.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" ");
        if expr.is_empty() {
            return Ok(Vec::new());
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.json FROM msg_fts JOIN messages m ON m.id=msg_fts.id \
                 WHERE msg_fts MATCH ?1 ORDER BY rank LIMIT ?2",
            )
            .map_err(map)?;
        let rows = stmt
            .query_map(rusqlite::params![expr, limit], |row| {
                row.get::<_, String>(0)
            })
            .map_err(map)?;
        let mut out = Vec::new();
        for r in rows {
            if let Ok(m) = serde_json::from_str::<MessageMetadata>(&r.map_err(map)?) {
                out.push(m);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: &str, label: &str, unread: i64, time: i64) -> MessageMetadata {
        MessageMetadata {
            id: id.into(),
            label_ids: vec![label.into()],
            unread,
            time,
            ..Default::default()
        }
    }

    #[test]
    fn upsert_list_delete_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("ptest-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        let c = Cache::open(&tmp).unwrap();

        assert_eq!(c.last_event_id().unwrap(), None);
        c.set_last_event_id("evt-1").unwrap();
        assert_eq!(c.last_event_id().unwrap().as_deref(), Some("evt-1"));

        c.upsert_message(&meta("a", "0", 1, 100)).unwrap();
        c.upsert_message(&meta("b", "0", 0, 200)).unwrap();
        let inbox = c.list("0", false, 10, 0).unwrap();
        assert_eq!(inbox.len(), 2);
        assert_eq!(inbox[0].id, "b"); // newest first
        assert_eq!(c.list("0", true, 10, 0).unwrap().len(), 1); // only unread
        assert_eq!(c.count("0").unwrap(), (2, 1));

        c.delete_message("a").unwrap();
        assert_eq!(c.list("0", false, 10, 0).unwrap().len(), 1);
        c.clear().unwrap();
        assert_eq!(c.list("0", false, 10, 0).unwrap().len(), 0);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn fts_index_and_search() {
        let tmp = std::env::temp_dir().join(format!("pfts-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        let c = Cache::open(&tmp).unwrap();
        c.index_message(
            &meta("m1", "5", 0, 10),
            "hello from protonmail-cli rust client",
        )
        .unwrap();
        c.index_message(&meta("m2", "5", 0, 20), "completely unrelated content here")
            .unwrap();
        let hits = c.search("protonmail-cli", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "m1");
        assert!(c.search("nonexistentword", 10).unwrap().is_empty());
        // multi-token AND
        assert_eq!(c.search("hello rust", 10).unwrap().len(), 1);
        let _ = std::fs::remove_file(&tmp);
    }
}
