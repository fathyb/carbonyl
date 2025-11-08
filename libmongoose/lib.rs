// === Existing Modules ===
pub mod browser;
pub mod cli;
pub mod gfx;
pub mod input;
pub mod output;
pub mod ui;

mod utils;

// === Phase 3: New Modules ===

/// Privacy engine - Ad and tracker blocking using adblock-rust
pub mod privacy;

/// ASCII art rendering - High-fidelity image-to-character conversion using chafa
pub mod render;

/// Data persistence - Hybrid SQLite + Redb storage
pub mod data;

/// Mojo IPC implementation - Handlers for mongoose.mojom interface
pub mod mojo_impl;

// Re-export key types for convenience
pub use mojo_impl::MongooseOrchestrator;
pub use privacy::{BlockResult, NetworkRequest, PrivacyEngine, RequestType};
pub use render::{AsciiEngine, FrameBuffer, RenderMode};
pub use data::{DataStore, HistoryEntry, Bookmark};
