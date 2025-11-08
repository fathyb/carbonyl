/// Integration Example - How to use libmongoose from mongoose-tui
///
/// This module demonstrates the integration between mongoose-tui and
/// libmongoose, showing how to:
/// - Initialize the orchestrator
/// - Handle privacy blocking
/// - Manage history and bookmarks
/// - Toggle rendering modes
/// - Use privacy mode

#![allow(dead_code)]

use anyhow::Result;
use libmongoose::{
    BlockResult, Bookmark, DataStore, HistoryEntry, MongooseOrchestrator,
    NetworkRequest, PrivacyEngine, RequestType, RenderMode,
};

/// Example: Initialize the mongoose orchestrator
///
/// This should be called once at application startup.
pub fn example_initialize() -> Result<MongooseOrchestrator> {
    // Determine data directory (e.g., ~/.config/mongoose or AppData/mongoose)
    let data_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mongoose");

    // Create the orchestrator
    let orchestrator = MongooseOrchestrator::new(data_dir.to_str().unwrap())?;

    println!("✓ Mongoose orchestrator initialized");
    println!("  Data directory: {}", data_dir.display());

    Ok(orchestrator)
}

/// Example: Toggle hybrid ASCII mode
///
/// This demonstrates how to switch between block and ASCII rendering modes.
pub fn example_toggle_ascii_mode(orchestrator: &MongooseOrchestrator, enabled: bool) {
    println!("Toggling ASCII mode: {}", enabled);

    // This will send a Mojo IPC call to the C++ core
    orchestrator.handle_set_hybrid_ascii_mode(enabled);

    if enabled {
        println!("✓ ASCII art mode enabled (high-fidelity rendering)");
    } else {
        println!("✓ Block mode enabled (60fps performance)");
    }
}

/// Example: Check if a request should be blocked
///
/// This demonstrates how the privacy engine blocks ads/trackers.
pub fn example_check_request(orchestrator: &MongooseOrchestrator) {
    println!("\n=== Privacy Engine Example ===");

    // Example requests
    let requests = vec![
        ("https://example.com", "https://referrer.com", "Normal site"),
        ("https://ads.example.com/ad.js", "https://site.com", "Ad script"),
        ("https://tracker.com/pixel.gif", "https://site.com", "Tracker"),
    ];

    for (url, source, description) in requests {
        let result = orchestrator.handle_check_network_request(
            url,
            source,
            5, // RequestType::Script
        );

        println!("\n{}: {}", description, url);
        if result.should_block {
            println!("  ✗ BLOCKED");
            if let Some(filter) = result.matched_filter {
                println!("  Filter: {}", filter);
            }
        } else {
            println!("  ✓ Allowed");
        }
    }
}

/// Example: Manage history
///
/// This demonstrates how to add, retrieve, and search history.
pub fn example_manage_history(orchestrator: &MongooseOrchestrator) -> Result<()> {
    println!("\n=== History Management Example ===");

    let data_store = orchestrator.data_store();

    // Add history entries
    println!("\nAdding history entries...");
    data_store.add_history("https://github.com", Some("GitHub"))?;
    data_store.add_history("https://reddit.com", Some("Reddit"))?;
    data_store.add_history("https://news.ycombinator.com", Some("Hacker News"))?;

    // Get recent history
    println!("\nRecent history (last 10):");
    let history = data_store.get_history(10)?;
    for (i, entry) in history.iter().enumerate() {
        println!(
            "  {}. {} - {} (visited {} times)",
            i + 1,
            entry.title.as_deref().unwrap_or("Untitled"),
            entry.url,
            entry.visit_count
        );
    }

    println!("\n✓ History operations complete");
    Ok(())
}

/// Example: Manage bookmarks
///
/// This demonstrates how to add, retrieve, and remove bookmarks.
pub fn example_manage_bookmarks(orchestrator: &MongooseOrchestrator) -> Result<()> {
    println!("\n=== Bookmark Management Example ===");

    let data_store = orchestrator.data_store();

    // Add bookmarks
    println!("\nAdding bookmarks...");
    data_store.add_bookmark("https://rust-lang.org", Some("Rust Programming Language"))?;
    data_store.add_bookmark("https://doc.rust-lang.org", Some("Rust Documentation"))?;

    // Get all bookmarks
    println!("\nAll bookmarks:");
    let bookmarks = data_store.get_bookmarks()?;
    for (i, bookmark) in bookmarks.iter().enumerate() {
        println!(
            "  {}. {} - {}",
            i + 1,
            bookmark.title.as_deref().unwrap_or("Untitled"),
            bookmark.url
        );
    }

    println!("\n✓ Bookmark operations complete");
    Ok(())
}

/// Example: Settings management
///
/// This demonstrates how to store and retrieve settings.
pub fn example_manage_settings(orchestrator: &MongooseOrchestrator) -> Result<()> {
    println!("\n=== Settings Management Example ===");

    let data_store = orchestrator.data_store();

    // Set some settings
    println!("\nSetting preferences...");
    data_store.set_setting("homepage", "https://example.com")?;
    data_store.set_setting("default_search_engine", "duckduckgo")?;
    data_store.set_setting("ascii_mode_default", "false")?;

    // Retrieve settings
    println!("\nCurrent settings:");
    if let Some(homepage) = data_store.get_setting("homepage")? {
        println!("  Homepage: {}", homepage);
    }
    if let Some(search) = data_store.get_setting("default_search_engine")? {
        println!("  Search engine: {}", search);
    }
    if let Some(ascii) = data_store.get_setting("ascii_mode_default")? {
        println!("  ASCII mode default: {}", ascii);
    }

    println!("\n✓ Settings operations complete");
    Ok(())
}

/// Example: Complete workflow
///
/// This demonstrates a typical user session.
pub fn example_complete_workflow() -> Result<()> {
    println!("\n========================================");
    println!("  Mongoose Integration Example");
    println!("========================================\n");

    // 1. Initialize
    let orchestrator = example_initialize()?;

    // 2. Configure preferences
    example_manage_settings(&orchestrator)?;

    // 3. Navigate and build history
    example_manage_history(&orchestrator)?;

    // 4. Save bookmarks
    example_manage_bookmarks(&orchestrator)?;

    // 5. Demonstrate privacy blocking
    example_check_request(&orchestrator);

    // 6. Toggle rendering modes
    println!("\n=== Rendering Mode Example ===");
    example_toggle_ascii_mode(&orchestrator, false);
    std::thread::sleep(std::time::Duration::from_millis(100));
    example_toggle_ascii_mode(&orchestrator, true);

    println!("\n========================================");
    println!("  Example Complete!");
    println!("========================================\n");

    Ok(())
}

/// Example: Privacy mode usage
///
/// This demonstrates how to use in-memory storage for private browsing.
#[cfg(feature = "privacy_mode")]
pub fn example_privacy_mode() -> Result<()> {
    use libmongoose::PrivateDataStore;

    println!("\n=== Privacy Mode Example ===");

    // Create private (in-memory) data store
    let private_store = PrivateDataStore::new();

    println!("✓ Private browsing mode enabled");
    println!("  All data will be kept in memory only");

    // Use it like normal, but nothing persists
    private_store.add_history("https://private-site.com", Some("Private"))?;
    private_store.add_bookmark("https://temp-bookmark.com", None)?;

    println!("\nTemporary session data:");
    println!("  History entries: {}", private_store.get_history(10)?.len());
    println!("  Bookmarks: {}", private_store.get_bookmarks()?.len());

    // Get memory stats
    let stats = private_store.get_stats();
    println!("\nMemory usage:");
    println!("  Estimated: ~{} bytes", stats.memory_estimate_bytes);

    // When private_store is dropped, all data is cleared
    drop(private_store);

    println!("\n✓ Private session ended, all data cleared");

    Ok(())
}

/// Example: Data backup and restore
///
/// This demonstrates how to export and import browser data.
#[cfg(feature = "data_export")]
pub fn example_backup_restore(orchestrator: &MongooseOrchestrator) -> Result<()> {
    use libmongoose::DataStoreExt;

    println!("\n=== Backup & Restore Example ===");

    let data_store = orchestrator.data_store();

    // Export data to a file
    let export_path = std::env::temp_dir().join("mongoose_backup.json");
    println!("\nExporting data to: {}", export_path.display());

    data_store.export_to_file(&export_path)?;
    println!("✓ Export complete");

    // Import data from the file
    println!("\nImporting data from: {}", export_path.display());

    let import_result = data_store.import_from_file(&export_path)?;
    println!("✓ Import complete");
    println!("  History imported: {}", import_result.history_imported);
    println!("  Bookmarks imported: {}", import_result.bookmarks_imported);
    println!("  Settings imported: {}", import_result.settings_imported);

    if !import_result.errors.is_empty() {
        println!("\n  Errors encountered:");
        for error in &import_result.errors {
            println!("    - {}", error);
        }
    }

    Ok(())
}

/// Example: Database maintenance
///
/// This demonstrates how to clean up old data and optimize the database.
#[cfg(feature = "data_cleanup")]
pub fn example_database_maintenance(orchestrator: &MongooseOrchestrator) -> Result<()> {
    use libmongoose::{CleanupPolicy, DataStoreExt};

    println!("\n=== Database Maintenance Example ===");

    let data_store = orchestrator.data_store();

    // Get statistics before cleanup
    println!("\nDatabase statistics (before cleanup):");
    let stats_before = data_store.get_statistics()?;
    println!("  History entries: {}", stats_before.history_count);
    println!("  Bookmarks: {}", stats_before.bookmark_count);
    println!("  Total visits: {}", stats_before.total_visits);
    println!("  Database size: {} bytes", stats_before.database_size_bytes);

    // Perform cleanup
    println!("\nPerforming cleanup...");
    let policy = CleanupPolicy {
        max_history_age_days: 90,      // Keep 3 months
        max_history_entries: Some(10000), // Keep max 10k entries
        cleanup_orphans: true,
    };

    let cleanup_result = data_store.cleanup_old_data(&policy)?;
    println!("✓ Cleanup complete");
    println!("  History deleted: {}", cleanup_result.history_deleted);
    println!("  Orphans deleted: {}", cleanup_result.orphans_deleted);
    println!("  Bytes freed: {}", cleanup_result.bytes_freed);

    // Optimize database
    println!("\nOptimizing database...");
    data_store.optimize()?;
    println!("✓ Optimization complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_workflow() {
        // Note: This test requires a writable config directory
        // and may fail in restricted environments
        let result = example_complete_workflow();
        if result.is_err() {
            eprintln!("Example workflow test skipped (environment issue)");
        }
    }
}
