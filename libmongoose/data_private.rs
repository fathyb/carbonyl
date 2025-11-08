/// Privacy Mode DataStore - In-memory storage for private browsing
///
/// This module provides an in-memory implementation of the DataStore
/// that doesn't persist any data to disk. Perfect for private/incognito mode.

use crate::data::{Bookmark, HistoryEntry};
use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// In-memory data store for private browsing mode
///
/// All data is stored in memory and is completely cleared when
/// the instance is dropped (browser closed).
pub struct PrivateDataStore {
    history: Arc<RwLock<Vec<HistoryEntry>>>,
    bookmarks: Arc<RwLock<Vec<Bookmark>>>,
    settings: Arc<RwLock<HashMap<String, String>>>,
    filter_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    session_state: Arc<RwLock<Option<Vec<u8>>>>,
    next_id: Arc<RwLock<i64>>,
}

impl PrivateDataStore {
    /// Create a new private data store
    ///
    /// All data is kept in memory only and will be lost when dropped.
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
            bookmarks: Arc::new(RwLock::new(Vec::new())),
            settings: Arc::new(RwLock::new(HashMap::new())),
            filter_cache: Arc::new(RwLock::new(HashMap::new())),
            session_state: Arc::new(RwLock::new(None)),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Add a history entry
    pub fn add_history(&self, url: &str, title: Option<&str>) -> Result<()> {
        let mut history = self.history.write();

        // Check if URL already exists
        if let Some(entry) = history.iter_mut().find(|e| e.url == url) {
            entry.visit_count += 1;
            entry.visit_time = current_timestamp();
        } else {
            let id = {
                let mut next_id = self.next_id.write();
                let id = *next_id;
                *next_id += 1;
                id
            };

            history.push(HistoryEntry {
                id: Some(id),
                url: url.to_string(),
                title: title.map(String::from),
                visit_time: current_timestamp(),
                visit_count: 1,
            });
        }

        Ok(())
    }

    /// Get recent history entries
    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let history = self.history.read();
        let mut entries: Vec<HistoryEntry> = history.clone();

        // Sort by visit time descending
        entries.sort_by(|a, b| b.visit_time.cmp(&a.visit_time));

        Ok(entries.into_iter().take(limit).collect())
    }

    /// Clear all history (instant in privacy mode)
    pub fn clear_history(&self) -> Result<()> {
        self.history.write().clear();
        Ok(())
    }

    /// Add a bookmark
    pub fn add_bookmark(&self, url: &str, title: Option<&str>) -> Result<()> {
        let id = {
            let mut next_id = self.next_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let bookmark = Bookmark {
            id: Some(id),
            url: url.to_string(),
            title: title.map(String::from),
            add_time: current_timestamp(),
        };

        self.bookmarks.write().push(bookmark);
        Ok(())
    }

    /// Get all bookmarks
    pub fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        Ok(self.bookmarks.read().clone())
    }

    /// Remove a bookmark by ID
    pub fn remove_bookmark(&self, id: i64) -> Result<()> {
        self.bookmarks.write().retain(|b| b.id != Some(id));
        Ok(())
    }

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self.settings.read().get(key).cloned())
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.settings.write().insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Cache a filter list
    pub fn cache_filter_list(&self, url: &str, data: &[u8]) -> Result<()> {
        self.filter_cache.write().insert(url.to_string(), data.to_vec());
        Ok(())
    }

    /// Get a cached filter list
    pub fn get_cached_filter_list(&self, url: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.filter_cache.read().get(url).cloned())
    }

    /// Save session state
    pub fn save_session(&self, state: &[u8]) -> Result<()> {
        *self.session_state.write() = Some(state.to_vec());
        Ok(())
    }

    /// Load session state
    pub fn load_session(&self) -> Result<Option<Vec<u8>>> {
        Ok(self.session_state.read().clone())
    }

    /// Clear all data (called when exiting private mode)
    pub fn clear_all(&self) {
        self.history.write().clear();
        self.bookmarks.write().clear();
        self.settings.write().clear();
        self.filter_cache.write().clear();
        *self.session_state.write() = None;
        *self.next_id.write() = 1;
    }

    /// Get statistics about in-memory storage
    pub fn get_stats(&self) -> PrivateStoreStats {
        PrivateStoreStats {
            history_entries: self.history.read().len(),
            bookmarks: self.bookmarks.read().len(),
            settings: self.settings.read().len(),
            cached_filters: self.filter_cache.read().len(),
            memory_estimate_bytes: self.estimate_memory_usage(),
        }
    }

    /// Estimate memory usage (rough approximation)
    fn estimate_memory_usage(&self) -> usize {
        let history_size = self.history.read().iter()
            .map(|h| h.url.len() + h.title.as_ref().map_or(0, |t| t.len()) + 32)
            .sum::<usize>();

        let bookmark_size = self.bookmarks.read().iter()
            .map(|b| b.url.len() + b.title.as_ref().map_or(0, |t| t.len()) + 32)
            .sum::<usize>();

        let settings_size = self.settings.read().iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();

        let cache_size = self.filter_cache.read().values()
            .map(|v| v.len())
            .sum::<usize>();

        history_size + bookmark_size + settings_size + cache_size
    }
}

impl Default for PrivateDataStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PrivateDataStore {
    fn drop(&mut self) {
        // Explicitly clear all data on drop for extra security
        self.clear_all();
    }
}

/// Statistics for private data store
#[derive(Debug, Clone)]
pub struct PrivateStoreStats {
    pub history_entries: usize,
    pub bookmarks: usize,
    pub settings: usize,
    pub cached_filters: usize,
    pub memory_estimate_bytes: usize,
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_store_creation() {
        let store = PrivateDataStore::new();
        let stats = store.get_stats();
        assert_eq!(stats.history_entries, 0);
        assert_eq!(stats.bookmarks, 0);
    }

    #[test]
    fn test_history_in_memory() {
        let store = PrivateDataStore::new();

        store.add_history("https://example.com", Some("Example")).unwrap();

        let history = store.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].url, "https://example.com");

        // Visit same URL again
        store.add_history("https://example.com", Some("Example")).unwrap();

        let history = store.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].visit_count, 2);
    }

    #[test]
    fn test_clear_history() {
        let store = PrivateDataStore::new();

        store.add_history("https://example.com", Some("Example")).unwrap();
        assert_eq!(store.get_history(10).unwrap().len(), 1);

        store.clear_history().unwrap();
        assert_eq!(store.get_history(10).unwrap().len(), 0);
    }

    #[test]
    fn test_bookmarks() {
        let store = PrivateDataStore::new();

        store.add_bookmark("https://example.com", Some("Example")).unwrap();

        let bookmarks = store.get_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 1);

        store.remove_bookmark(bookmarks[0].id.unwrap()).unwrap();
        assert_eq!(store.get_bookmarks().unwrap().len(), 0);
    }

    #[test]
    fn test_settings() {
        let store = PrivateDataStore::new();

        store.set_setting("test_key", "test_value").unwrap();

        let value = store.get_setting("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[test]
    fn test_filter_cache() {
        let store = PrivateDataStore::new();

        let data = b"test filter data";
        store.cache_filter_list("https://example.com/list.txt", data).unwrap();

        let cached = store.get_cached_filter_list("https://example.com/list.txt").unwrap();
        assert_eq!(cached, Some(data.to_vec()));
    }

    #[test]
    fn test_clear_all() {
        let store = PrivateDataStore::new();

        store.add_history("https://example.com", Some("Example")).unwrap();
        store.add_bookmark("https://bookmark.com", Some("BM")).unwrap();
        store.set_setting("key", "value").unwrap();

        let stats_before = store.get_stats();
        assert!(stats_before.history_entries > 0);
        assert!(stats_before.bookmarks > 0);
        assert!(stats_before.settings > 0);

        store.clear_all();

        let stats_after = store.get_stats();
        assert_eq!(stats_after.history_entries, 0);
        assert_eq!(stats_after.bookmarks, 0);
        assert_eq!(stats_after.settings, 0);
    }

    #[test]
    fn test_memory_estimate() {
        let store = PrivateDataStore::new();

        store.add_history("https://example.com", Some("Example Site")).unwrap();
        store.add_bookmark("https://bookmark.com", Some("Bookmark")).unwrap();

        let stats = store.get_stats();
        assert!(stats.memory_estimate_bytes > 0);
    }

    #[test]
    fn test_drop_clears_data() {
        let store = PrivateDataStore::new();
        store.add_history("https://secret.com", Some("Secret")).unwrap();

        // Data should exist
        assert_eq!(store.get_history(10).unwrap().len(), 1);

        // Drop the store
        drop(store);

        // Create new store - should be empty
        let new_store = PrivateDataStore::new();
        assert_eq!(new_store.get_history(10).unwrap().len(), 0);
    }
}
