/// Data Persistence Layer - Hybrid SQLite + Redb storage
///
/// This module provides persistent storage for:
/// - Relational data (history, bookmarks, settings) using rusqlite
/// - Blob/cache data (filter lists, session state) using redb

use anyhow::{Context, Result};
use log::{debug, info};
use parking_lot::Mutex;
use redb::{Database, ReadableTable, TableDefinition};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default database paths (relative to config directory)
const SQLITE_DB_NAME: &str = "mongoose.sqlite";
const REDB_DB_NAME: &str = "cache.redb";

/// Redb table definitions
const ADBLOCK_LISTS: TableDefinition<&str, &str> = TableDefinition::new("adblock_lists");
const ADBLOCK_CACHE: TableDefinition<&str, &[u8]> = TableDefinition::new("adblock_cache");
const SESSION_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("session");

/// A history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Option<i64>,
    pub url: String,
    pub title: Option<String>,
    pub visit_time: i64,
    pub visit_count: i32,
}

/// A bookmark entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: Option<i64>,
    pub url: String,
    pub title: Option<String>,
    pub add_time: i64,
}

/// Site-specific permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitePermissions {
    pub origin: String,
    pub javascript: bool,
    pub cookies: bool,
    pub images: bool,
}

/// The hybrid datastore
pub struct DataStore {
    sqlite: Arc<Mutex<Connection>>,
    redb: Arc<Database>,
    data_dir: PathBuf,
}

impl DataStore {
    /// Create or open the datastore
    ///
    /// # Arguments
    /// * `data_dir` - Directory where database files will be stored
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();

        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;

        // Open SQLite database
        let sqlite_path = data_dir.join(SQLITE_DB_NAME);
        let sqlite = Connection::open(&sqlite_path)
            .context("Failed to open SQLite database")?;

        // Initialize SQLite schema
        Self::init_sqlite_schema(&sqlite)?;

        // Open Redb database
        let redb_path = data_dir.join(REDB_DB_NAME);
        let redb = Database::create(&redb_path)
            .context("Failed to open Redb database")?;

        // Initialize Redb tables
        Self::init_redb_schema(&redb)?;

        info!("DataStore initialized at: {}", data_dir.display());

        Ok(Self {
            sqlite: Arc::new(Mutex::new(sqlite)),
            redb: Arc::new(redb),
            data_dir,
        })
    }

    /// Initialize SQLite schema
    fn init_sqlite_schema(conn: &Connection) -> Result<()> {
        // History table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT,
                visit_time INTEGER NOT NULL,
                visit_count INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        // Bookmarks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT,
                add_time INTEGER NOT NULL
            )",
            [],
        )?;

        // Site permissions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS site_permissions (
                origin TEXT PRIMARY KEY,
                javascript INTEGER NOT NULL DEFAULT 1,
                cookies INTEGER NOT NULL DEFAULT 1,
                images INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        // Settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )?;

        // Create indices for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_time ON history(visit_time)",
            [],
        )?;

        debug!("SQLite schema initialized");
        Ok(())
    }

    /// Initialize Redb schema
    fn init_redb_schema(db: &Database) -> Result<()> {
        let write_txn = db.begin_write()?;

        // Create tables (they're created automatically on first use)
        {
            let _adblock_lists = write_txn.open_table(ADBLOCK_LISTS)?;
            let _adblock_cache = write_txn.open_table(ADBLOCK_CACHE)?;
            let _session = write_txn.open_table(SESSION_TABLE)?;
        }

        write_txn.commit()?;

        debug!("Redb schema initialized");
        Ok(())
    }

    // === History Management ===

    /// Add a history entry
    pub fn add_history(&self, url: &str, title: Option<&str>) -> Result<()> {
        let conn = self.sqlite.lock();
        let now = Self::current_timestamp();

        // Check if URL already exists
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM history WHERE url = ?",
                params![url],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing {
            // Update visit count
            conn.execute(
                "UPDATE history SET visit_count = visit_count + 1, visit_time = ? WHERE id = ?",
                params![now, id],
            )?;
        } else {
            // Insert new entry
            conn.execute(
                "INSERT INTO history (url, title, visit_time, visit_count) VALUES (?, ?, ?, 1)",
                params![url, title, now],
            )?;
        }

        debug!("Added history entry: {}", url);
        Ok(())
    }

    /// Get recent history entries
    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let conn = self.sqlite.lock();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, visit_time, visit_count FROM history
             ORDER BY visit_time DESC LIMIT ?"
        )?;

        let entries = stmt
            .query_map(params![limit], |row| {
                Ok(HistoryEntry {
                    id: Some(row.get(0)?),
                    url: row.get(1)?,
                    title: row.get(2)?,
                    visit_time: row.get(3)?,
                    visit_count: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Clear all history
    pub fn clear_history(&self) -> Result<()> {
        let conn = self.sqlite.lock();
        conn.execute("DELETE FROM history", [])?;
        info!("History cleared");
        Ok(())
    }

    // === Bookmark Management ===

    /// Add a bookmark
    pub fn add_bookmark(&self, url: &str, title: Option<&str>) -> Result<()> {
        let conn = self.sqlite.lock();
        let now = Self::current_timestamp();

        conn.execute(
            "INSERT INTO bookmarks (url, title, add_time) VALUES (?, ?, ?)",
            params![url, title, now],
        )?;

        info!("Added bookmark: {}", url);
        Ok(())
    }

    /// Get all bookmarks
    pub fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let conn = self.sqlite.lock();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, add_time FROM bookmarks ORDER BY add_time DESC"
        )?;

        let bookmarks = stmt
            .query_map([], |row| {
                Ok(Bookmark {
                    id: Some(row.get(0)?),
                    url: row.get(1)?,
                    title: row.get(2)?,
                    add_time: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(bookmarks)
    }

    /// Remove a bookmark by ID
    pub fn remove_bookmark(&self, id: i64) -> Result<()> {
        let conn = self.sqlite.lock();
        conn.execute("DELETE FROM bookmarks WHERE id = ?", params![id])?;
        info!("Removed bookmark ID: {}", id);
        Ok(())
    }

    // === Settings Management ===

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.sqlite.lock();

        let value: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                params![key],
                |row| row.get(0),
            )
            .ok();

        Ok(value)
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.sqlite.lock();

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, value],
        )?;

        debug!("Set setting: {} = {}", key, value);
        Ok(())
    }

    // === Ad-block Filter List Cache (Redb) ===

    /// Cache a filter list
    pub fn cache_filter_list(&self, url: &str, data: &[u8]) -> Result<()> {
        let write_txn = self.redb.begin_write()?;

        {
            let mut table = write_txn.open_table(ADBLOCK_CACHE)?;
            table.insert(url, data)?;
        }

        write_txn.commit()?;

        debug!("Cached filter list: {} ({} bytes)", url, data.len());
        Ok(())
    }

    /// Get a cached filter list
    pub fn get_cached_filter_list(&self, url: &str) -> Result<Option<Vec<u8>>> {
        let read_txn = self.redb.begin_read()?;
        let table = read_txn.open_table(ADBLOCK_CACHE)?;

        match table.get(url)? {
            Some(data) => Ok(Some(data.value().to_vec())),
            None => Ok(None),
        }
    }

    // === Session State (Redb) ===

    /// Save session state (open tabs, etc.)
    pub fn save_session(&self, state: &[u8]) -> Result<()> {
        let write_txn = self.redb.begin_write()?;

        {
            let mut table = write_txn.open_table(SESSION_TABLE)?;
            table.insert("current_session", state)?;
        }

        write_txn.commit()?;

        debug!("Saved session state ({} bytes)", state.len());
        Ok(())
    }

    /// Load session state
    pub fn load_session(&self) -> Result<Option<Vec<u8>>> {
        let read_txn = self.redb.begin_read()?;
        let table = read_txn.open_table(SESSION_TABLE)?;

        match table.get("current_session")? {
            Some(data) => Ok(Some(data.value().to_vec())),
            None => Ok(None),
        }
    }

    // === Utilities ===

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_datastore_creation() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path());
        assert!(store.is_ok());
    }

    #[test]
    fn test_history_management() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path()).unwrap();

        // Add history
        store.add_history("https://example.com", Some("Example")).unwrap();

        // Get history
        let history = store.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].url, "https://example.com");

        // Clear history
        store.clear_history().unwrap();
        let history = store.get_history(10).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_bookmark_management() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path()).unwrap();

        // Add bookmark
        store.add_bookmark("https://example.com", Some("Example")).unwrap();

        // Get bookmarks
        let bookmarks = store.get_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://example.com");

        // Remove bookmark
        store.remove_bookmark(bookmarks[0].id.unwrap()).unwrap();
        let bookmarks = store.get_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 0);
    }

    #[test]
    fn test_settings() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path()).unwrap();

        // Set a setting
        store.set_setting("homepage", "https://example.com").unwrap();

        // Get the setting
        let value = store.get_setting("homepage").unwrap();
        assert_eq!(value, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_filter_list_cache() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path()).unwrap();

        let url = "https://example.com/list.txt";
        let data = b"test filter list data";

        // Cache the list
        store.cache_filter_list(url, data).unwrap();

        // Retrieve the list
        let cached = store.get_cached_filter_list(url).unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), data);
    }
}
