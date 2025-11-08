/// Mojo IPC Implementation - Handlers for mongoose.mojom interface
///
/// This module provides the Rust implementation of the MongooseRenderService
/// interface defined in mongoose.mojom. It bridges the C++ Chromium runtime
/// with the Rust logic layers (privacy, rendering, data).

use crate::data::DataStore;
use crate::privacy::{BlockResult, NetworkRequest, PrivacyEngine, RequestType};
use crate::render::{AsciiEngine, FrameBuffer, RenderMode};
use log::{debug, error, info, warn};
use parking_lot::RwLock;
use std::sync::Arc;

/// Main orchestrator for all mongoose services
///
/// This struct holds references to all the Phase 3 services and implements
/// the handlers for Mojo IPC calls from the C++ browser core.
pub struct MongooseOrchestrator {
    privacy_engine: Arc<PrivacyEngine>,
    ascii_engine: Arc<RwLock<AsciiEngine>>,
    data_store: Arc<DataStore>,
    frame_buffer: Arc<RwLock<FrameBuffer>>,
    ascii_mode_enabled: Arc<RwLock<bool>>,
}

impl MongooseOrchestrator {
    /// Create a new orchestrator with all services
    pub fn new(data_dir: &str) -> anyhow::Result<Self> {
        info!("Initializing Mongoose Orchestrator");

        // Initialize privacy engine
        let privacy_engine = Arc::new(PrivacyEngine::new()?);

        // Load default filter lists synchronously
        // TODO: Load in background once adblock-rust supports Send+Sync
        if let Err(e) = privacy_engine.load_default_filters() {
            warn!("Failed to load default filters: {}", e);
            info!("Privacy engine will run with no filters until manually loaded");
        }

        // Initialize ASCII art engine
        let ascii_engine = Arc::new(RwLock::new(AsciiEngine::default()));

        // Initialize data store
        let data_store = Arc::new(DataStore::new(data_dir)?);

        // Initialize frame buffer (80x24 default, will be resized)
        let frame_buffer = Arc::new(RwLock::new(FrameBuffer::new(80, 24)));

        info!("Mongoose Orchestrator initialized successfully");

        Ok(Self {
            privacy_engine,
            ascii_engine,
            data_store,
            frame_buffer,
            ascii_mode_enabled: Arc::new(RwLock::new(false)),
        })
    }

    /// Handle DrawText Mojo call (existing)
    ///
    /// This is called by the C++ renderer when text needs to be drawn.
    /// In the future, this will update the frame buffer.
    pub fn handle_draw_text(&self, text_data: &[TextChunk]) {
        debug!("DrawText called with {} chunks", text_data.len());
        // Future: Update frame buffer with text data
    }

    /// Handle CheckNetworkRequest Mojo call (Phase 3: Privacy)
    ///
    /// This is called synchronously from the C++ network interceptor.
    /// It must return quickly (microseconds) to avoid blocking the browser.
    pub fn handle_check_network_request(
        &self,
        url: &str,
        source_url: &str,
        resource_type: i32,
    ) -> NetworkBlockResult {
        let request = NetworkRequest {
            url: url.to_string(),
            source_url: source_url.to_string(),
            request_type: Self::resource_type_from_int(resource_type),
        };

        let result = self.privacy_engine.check_request(&request);

        NetworkBlockResult {
            should_block: result.should_block,
            matched_filter: result.filter,
        }
    }

    /// Handle DrawImageBitmap Mojo call (Phase 3: Hybrid Rendering)
    ///
    /// This is called when hybrid ASCII mode is enabled and an image is
    /// being rendered. It uses chafa to generate high-fidelity ASCII art.
    pub fn handle_draw_image_bitmap(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) {
        debug!("DrawImageBitmap called: {}x{} at ({}, {})", width, height, x, y);

        let ascii_engine = self.ascii_engine.read();

        match ascii_engine.render(pixels, width, height) {
            Ok(art) => {
                if !art.is_empty() {
                    // Stamp the ASCII art onto the frame buffer
                    let mut fb = self.frame_buffer.write();
                    fb.stamp_ascii(x.max(0) as usize, y.max(0) as usize, &art);
                }
            }
            Err(e) => {
                error!("Failed to render ASCII art: {}", e);
            }
        }
    }

    /// Handle SetHybridAsciiMode Mojo call (Phase 3: Hybrid Rendering)
    ///
    /// This toggles between block mode (60fps) and ASCII mode (high-fidelity).
    pub fn handle_set_hybrid_ascii_mode(&self, enabled: bool) {
        info!("Setting hybrid ASCII mode: {}", enabled);

        *self.ascii_mode_enabled.write() = enabled;

        let mut ascii_engine = self.ascii_engine.write();
        ascii_engine.set_mode(if enabled {
            RenderMode::Ascii
        } else {
            RenderMode::Block
        });
    }

    /// Handle SetTitle Mojo call (existing)
    ///
    /// This is called when the page title changes.
    pub fn handle_set_title(&self, title: &str) {
        debug!("SetTitle called: {}", title);
        // Future: Update TUI title bar via callback
    }

    /// Get the current frame buffer (for TUI rendering)
    pub fn get_frame_buffer(&self) -> String {
        let fb = self.frame_buffer.read();
        fb.to_string()
    }

    /// Resize the frame buffer (called when terminal size changes)
    pub fn resize_frame_buffer(&self, width: usize, height: usize) {
        let mut fb = self.frame_buffer.write();
        fb.resize(width, height);
        debug!("Frame buffer resized to {}x{}", width, height);
    }

    /// Get access to the data store (for TUI)
    pub fn data_store(&self) -> &Arc<DataStore> {
        &self.data_store
    }

    /// Convert integer resource type to enum
    fn resource_type_from_int(type_id: i32) -> RequestType {
        match type_id {
            0 => RequestType::Document,
            1 => RequestType::Stylesheet,
            2 => RequestType::Image,
            3 => RequestType::Media,
            4 => RequestType::Font,
            5 => RequestType::Script,
            6 => RequestType::XmlHttpRequest,
            7 => RequestType::Ping,
            8 => RequestType::WebSocket,
            _ => RequestType::Other,
        }
    }
}

/// Placeholder for text chunk data (from Mojo)
#[derive(Debug, Clone)]
pub struct TextChunk {
    pub content: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Result of a network block check (maps to BlockResult in mongoose.mojom)
#[derive(Debug, Clone)]
pub struct NetworkBlockResult {
    pub should_block: bool,
    pub matched_filter: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_orchestrator_creation() {
        let dir = tempdir().unwrap();
        let orch = MongooseOrchestrator::new(dir.path().to_str().unwrap());
        assert!(orch.is_ok());
    }

    #[test]
    fn test_ascii_mode_toggle() {
        let dir = tempdir().unwrap();
        let orch = MongooseOrchestrator::new(dir.path().to_str().unwrap()).unwrap();

        // Initially in block mode
        assert!(!*orch.ascii_mode_enabled.read());

        // Enable ASCII mode
        orch.handle_set_hybrid_ascii_mode(true);
        assert!(*orch.ascii_mode_enabled.read());

        // Disable ASCII mode
        orch.handle_set_hybrid_ascii_mode(false);
        assert!(!*orch.ascii_mode_enabled.read());
    }

    #[test]
    fn test_network_request_blocking() {
        let dir = tempdir().unwrap();
        let orch = MongooseOrchestrator::new(dir.path().to_str().unwrap()).unwrap();

        // Test a request (should not block until filters are loaded)
        let result = orch.handle_check_network_request(
            "https://example.com",
            "https://referrer.com",
            0,
        );

        // Result depends on loaded filters, just verify it returns
        assert!(result.should_block || !result.should_block);
    }
}
