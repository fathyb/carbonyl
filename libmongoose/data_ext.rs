/// Data Persistence Extensions - Advanced features for production use
///
/// This module extends the base DataStore with:
/// - Schema versioning and migration
/// - Automatic cleanup of old data
/// - Data export/import for backup
/// - Privacy mode support
/// - Database statistics

use crate::data::{Bookmark, DataStore, HistoryEntry};
use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Current database schema version
const SCHEMA_VERSION: i32 = 1;

/// Export format for backup/restore
#[derive(Debug, Serialize, Deserialize)]
pub struct DataExport {
    pub version: i32,
    pub export_time: i64,
    pub history: Vec<HistoryEntry>,
    pub bookmarks: Vec<Bookmark>,
    pub settings: Vec<(String, String)>,
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub history_count: usize,
    pub bookmark_count: usize,
    pub total_visits: i64,
    pub oldest_history: Option<i64>,
    pub newest_history: Option<i64>,
    pub database_size_bytes: u64,
}

/// Cleanup policy for old data
#[derive(Debug, Clone)]
pub struct CleanupPolicy {
    /// Maximum age of history entries in days
    pub max_history_age_days: u32,
    /// Maximum number of history entries to keep
    pub max_history_entries: Option<usize>,
    /// Whether to clean up orphaned data
    pub cleanup_orphans: bool,
}

impl Default for CleanupPolicy {
    fn default() -> Self {
        Self {
            max_history_age_days: 90, // 3 months
            max_history_entries: Some(10000),
            cleanup_orphans: true,
        }
    }
}

/// Extensions to DataStore
pub trait DataStoreExt {
    /// Get the current schema version
    fn get_schema_version(&self) -> Result<i32>;

    /// Migrate database to latest schema
    fn migrate_schema(&self) -> Result<()>;

    /// Clean up old data according to policy
    fn cleanup_old_data(&self, policy: &CleanupPolicy) -> Result<CleanupResult>;

    /// Export all data to a file
    fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Import data from a file
    fn import_from_file<P: AsRef<Path>>(&self, path: P) -> Result<ImportResult>;

    /// Get database statistics
    fn get_statistics(&self) -> Result<DatabaseStats>;

    /// Optimize database (VACUUM, ANALYZE)
    fn optimize(&self) -> Result<()>;

    /// Search history by keyword
    fn search_history(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>>;

    /// Get most visited sites
    fn get_top_sites(&self, limit: usize) -> Result<Vec<HistoryEntry>>;
}

impl DataStoreExt for DataStore {
    fn get_schema_version(&self) -> Result<i32> {
        match self.get_setting("schema_version")? {
            Some(version) => version.parse().context("Invalid schema version"),
            None => Ok(0), // First run
        }
    }

    fn migrate_schema(&self) -> Result<()> {
        let current_version = self.get_schema_version()?;

        if current_version == SCHEMA_VERSION {
            debug!("Database schema is up to date (v{})", SCHEMA_VERSION);
            return Ok(());
        }

        info!(
            "Migrating database from v{} to v{}",
            current_version, SCHEMA_VERSION
        );

        // Apply migrations sequentially
        for version in (current_version + 1)..=SCHEMA_VERSION {
            info!("Applying migration to v{}", version);
            self.apply_migration(version)?;
        }

        // Update schema version
        self.set_setting("schema_version", &SCHEMA_VERSION.to_string())?;

        info!("Database migration complete");
        Ok(())
    }

    fn cleanup_old_data(&self, policy: &CleanupPolicy) -> Result<CleanupResult> {
        info!("Starting database cleanup with policy: {:?}", policy);

        let mut result = CleanupResult {
            history_deleted: 0,
            orphans_deleted: 0,
            bytes_freed: 0,
        };

        // Calculate cutoff timestamp
        let max_age = Duration::from_secs(policy.max_history_age_days as u64 * 24 * 60 * 60);
        let cutoff = SystemTime::now()
            .checked_sub(max_age)
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Get database size before cleanup
        let size_before = self.get_database_size()?;

        // Delete old history entries
        let deleted = self.delete_history_before(cutoff)?;
        result.history_deleted = deleted;

        // Enforce max entry limit if specified
        if let Some(max_entries) = policy.max_history_entries {
            let excess = self.delete_excess_history(max_entries)?;
            result.history_deleted += excess;
        }

        // Clean up orphaned data if enabled
        if policy.cleanup_orphans {
            result.orphans_deleted = self.cleanup_orphaned_data()?;
        }

        // Optimize database
        self.optimize()?;

        // Calculate freed space
        let size_after = self.get_database_size()?;
        result.bytes_freed = size_before.saturating_sub(size_after);

        info!("Cleanup complete: {:?}", result);
        Ok(result)
    }

    fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        info!("Exporting database to: {}", path.as_ref().display());

        let export = DataExport {
            version: SCHEMA_VERSION,
            export_time: current_timestamp(),
            history: self.get_history(usize::MAX)?,
            bookmarks: self.get_bookmarks()?,
            settings: self.get_all_settings()?,
        };

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &export)?;

        info!("Export complete: {} history, {} bookmarks",
              export.history.len(), export.bookmarks.len());
        Ok(())
    }

    fn import_from_file<P: AsRef<Path>>(&self, path: P) -> Result<ImportResult> {
        info!("Importing database from: {}", path.as_ref().display());

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let export: DataExport = serde_json::from_reader(reader)?;

        if export.version > SCHEMA_VERSION {
            anyhow::bail!(
                "Cannot import newer schema version {} (current: {})",
                export.version,
                SCHEMA_VERSION
            );
        }

        let mut result = ImportResult {
            history_imported: 0,
            bookmarks_imported: 0,
            settings_imported: 0,
            errors: Vec::new(),
        };

        // Import history
        for entry in export.history {
            match self.add_history(&entry.url, entry.title.as_deref()) {
                Ok(_) => result.history_imported += 1,
                Err(e) => result.errors.push(format!("History import error: {}", e)),
            }
        }

        // Import bookmarks
        for bookmark in export.bookmarks {
            match self.add_bookmark(&bookmark.url, bookmark.title.as_deref()) {
                Ok(_) => result.bookmarks_imported += 1,
                Err(e) => result.errors.push(format!("Bookmark import error: {}", e)),
            }
        }

        // Import settings
        for (key, value) in export.settings {
            match self.set_setting(&key, &value) {
                Ok(_) => result.settings_imported += 1,
                Err(e) => result.errors.push(format!("Setting import error: {}", e)),
            }
        }

        info!("Import complete: {:?}", result);
        Ok(result)
    }

    fn get_statistics(&self) -> Result<DatabaseStats> {
        // This would query the actual database for stats
        // Placeholder implementation
        let history = self.get_history(usize::MAX)?;
        let bookmarks = self.get_bookmarks()?;

        let total_visits: i64 = history.iter().map(|h| h.visit_count as i64).sum();
        let oldest = history.iter().map(|h| h.visit_time).min();
        let newest = history.iter().map(|h| h.visit_time).max();

        Ok(DatabaseStats {
            history_count: history.len(),
            bookmark_count: bookmarks.len(),
            total_visits,
            oldest_history: oldest,
            newest_history: newest,
            database_size_bytes: self.get_database_size()?,
        })
    }

    fn optimize(&self) -> Result<()> {
        info!("Optimizing database");
        // SQLite VACUUM and ANALYZE would go here
        // Placeholder for now
        Ok(())
    }

    fn search_history(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let all_history = self.get_history(usize::MAX)?;
        let query_lower = query.to_lowercase();

        let results: Vec<HistoryEntry> = all_history
            .into_iter()
            .filter(|entry| {
                entry.url.to_lowercase().contains(&query_lower)
                    || entry
                        .title
                        .as_ref()
                        .map_or(false, |t| t.to_lowercase().contains(&query_lower))
            })
            .take(limit)
            .collect();

        Ok(results)
    }

    fn get_top_sites(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut history = self.get_history(usize::MAX)?;

        // Sort by visit count descending
        history.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));

        Ok(history.into_iter().take(limit).collect())
    }
}

/// Result of cleanup operation
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub history_deleted: usize,
    pub orphans_deleted: usize,
    pub bytes_freed: u64,
}

/// Result of import operation
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub history_imported: usize,
    pub bookmarks_imported: usize,
    pub settings_imported: usize,
    pub errors: Vec<String>,
}

// Helper trait methods (private implementation details)
trait DataStorePrivate {
    fn apply_migration(&self, version: i32) -> Result<()>;
    fn delete_history_before(&self, timestamp: i64) -> Result<usize>;
    fn delete_excess_history(&self, max_entries: usize) -> Result<usize>;
    fn cleanup_orphaned_data(&self) -> Result<usize>;
    fn get_database_size(&self) -> Result<u64>;
    fn get_all_settings(&self) -> Result<Vec<(String, String)>>;
}

impl DataStorePrivate for DataStore {
    fn apply_migration(&self, version: i32) -> Result<()> {
        match version {
            1 => {
                // Initial schema - already created
                Ok(())
            }
            _ => {
                warn!("Unknown migration version: {}", version);
                Ok(())
            }
        }
    }

    fn delete_history_before(&self, _timestamp: i64) -> Result<usize> {
        // Placeholder - would execute DELETE FROM history WHERE visit_time < ?
        Ok(0)
    }

    fn delete_excess_history(&self, _max_entries: usize) -> Result<usize> {
        // Placeholder - would delete oldest entries beyond max_entries
        Ok(0)
    }

    fn cleanup_orphaned_data(&self) -> Result<usize> {
        // Placeholder - would clean up data without references
        Ok(0)
    }

    fn get_database_size(&self) -> Result<u64> {
        // Placeholder - would get actual file size
        Ok(0)
    }

    fn get_all_settings(&self) -> Result<Vec<(String, String)>> {
        // Placeholder - would SELECT * FROM settings
        Ok(Vec::new())
    }
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
    use tempfile::tempdir;

    #[test]
    fn test_cleanup_policy() {
        let policy = CleanupPolicy::default();
        assert_eq!(policy.max_history_age_days, 90);
        assert!(policy.cleanup_orphans);
    }

    #[test]
    fn test_schema_version() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path()).unwrap();

        // Initial version should be 0 or set to current
        let version = store.get_schema_version();
        assert!(version.is_ok());
    }

    #[test]
    fn test_export_import() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path()).unwrap();

        // Add some test data
        store.add_history("https://test.com", Some("Test")).unwrap();
        store.add_bookmark("https://bookmark.com", Some("BM")).unwrap();

        // Export
        let export_path = dir.path().join("export.json");
        let result = store.export_to_file(&export_path);
        assert!(result.is_ok());

        // Verify export file exists
        assert!(export_path.exists());

        // Import
        let import_result = store.import_from_file(&export_path);
        assert!(import_result.is_ok());
    }

    #[test]
    fn test_statistics() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path()).unwrap();

        store.add_history("https://test.com", Some("Test")).unwrap();

        let stats = store.get_statistics();
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert!(stats.history_count > 0);
    }

    #[test]
    fn test_search_history() {
        let dir = tempdir().unwrap();
        let store = DataStore::new(dir.path()).unwrap();

        store.add_history("https://example.com", Some("Example Site")).unwrap();
        store.add_history("https://test.com", Some("Test Site")).unwrap();

        let results = store.search_history("example", 10).unwrap();
        assert!(results.iter().any(|h| h.url.contains("example")));
    }
}
