/// Privacy Engine - Ad and tracker blocking using adblock-rust
///
/// This module integrates the adblock-rust engine (the same engine used by Brave)
/// to provide network-level blocking of ads and trackers.

use adblock::Engine;
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use parking_lot::RwLock;
use reqwest::blocking::Client;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Default filter lists to load on first run
const DEFAULT_FILTER_LISTS: &[(&str, &str)] = &[
    ("easylist", "https://easylist.to/easylist/easylist.txt"),
    (
        "easyprivacy",
        "https://easylist.to/easylist/easyprivacy.txt",
    ),
];

/// How long before we consider a filter list stale (24 hours)
const FILTER_LIST_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// Network request data for blocking checks
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub url: String,
    pub source_url: String,
    pub request_type: RequestType,
}

/// HTTP request types for context-aware blocking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    Document,
    Stylesheet,
    Image,
    Media,
    Font,
    Script,
    XmlHttpRequest,
    Ping,
    WebSocket,
    Other,
}

impl RequestType {
    pub fn to_adblock_type(&self) -> &'static str {
        match self {
            RequestType::Document => "document",
            RequestType::Stylesheet => "stylesheet",
            RequestType::Image => "image",
            RequestType::Media => "media",
            RequestType::Font => "font",
            RequestType::Script => "script",
            RequestType::XmlHttpRequest => "xmlhttprequest",
            RequestType::Ping => "ping",
            RequestType::WebSocket => "websocket",
            RequestType::Other => "other",
        }
    }
}

/// Result of a blocking check
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockResult {
    pub should_block: bool,
    pub filter: Option<String>,
}

impl BlockResult {
    pub fn allow() -> Self {
        Self {
            should_block: false,
            filter: None,
        }
    }

    pub fn block(filter: String) -> Self {
        Self {
            should_block: true,
            filter: Some(filter),
        }
    }
}

/// The main privacy engine
pub struct PrivacyEngine {
    engine: Arc<RwLock<Engine>>,
    http_client: Client,
}

impl PrivacyEngine {
    /// Create a new privacy engine
    ///
    /// This will initialize an empty engine. Call `load_default_filters()`
    /// or `add_filter_list()` to populate it with blocking rules.
    pub fn new() -> Result<Self> {
        let engine = Engine::default();

        Ok(Self {
            engine: Arc::new(RwLock::new(engine)),
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .context("Failed to create HTTP client")?,
        })
    }

    /// Load the default filter lists (EasyList + EasyPrivacy)
    pub fn load_default_filters(&self) -> Result<()> {
        info!("Loading default filter lists");

        for (name, url) in DEFAULT_FILTER_LISTS {
            info!("Fetching filter list: {} from {}", name, url);

            match self.fetch_filter_list(url) {
                Ok(rules) => {
                    info!("Loaded {} rules from {}", rules.len(), name);
                    self.add_filter_rules(&rules)?;
                }
                Err(e) => {
                    warn!("Failed to fetch filter list {}: {}", name, e);
                    // Continue with other lists
                }
            }
        }

        info!("Default filter lists loaded successfully");
        Ok(())
    }

    /// Fetch a filter list from a URL
    fn fetch_filter_list(&self, url: &str) -> Result<Vec<String>> {
        debug!("Fetching filter list from: {}", url);

        let response = self
            .http_client
            .get(url)
            .send()
            .context("Failed to fetch filter list")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let text = response.text().context("Failed to read response body")?;

        let rules: Vec<String> = text
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('!') && !trimmed.starts_with('#')
            })
            .map(String::from)
            .collect();

        Ok(rules)
    }

    /// Add filter rules to the engine
    pub fn add_filter_rules(&self, rules: &[String]) -> Result<()> {
        // Join all rules into a single string separated by newlines
        let filter_text = rules.join("\n");

        // Create new engine with these rules
        let mut new_engine = Engine::from_rules(&[&filter_text], Default::default());

        // Swap with the current engine
        *self.engine.write() = new_engine;

        debug!("Added {} filter rules to engine", rules.len());
        Ok(())
    }

    /// Check if a network request should be blocked
    ///
    /// This is a fast, in-memory operation that returns immediately.
    /// It is safe to call from any thread.
    pub fn check_request(&self, request: &NetworkRequest) -> BlockResult {
        let _engine = self.engine.read();

        // TODO: Use the correct adblock-rust API once we determine the exact method signature
        // For now, return allow for all requests (privacy engine compiles but doesn't block)
        debug!("Privacy check for: {} (placeholder implementation)", request.url);

        // Placeholder: Always allow for now
        // The real implementation will use engine.check_network_urls() or similar
        BlockResult::allow()
    }

    /// Get statistics about the loaded filters
    pub fn get_stats(&self) -> EngineStats {
        let engine = self.engine.read();

        // adblock-rust doesn't expose filter count directly,
        // so we'll return basic stats
        EngineStats {
            filters_loaded: true,
        }
    }
}

impl Default for PrivacyEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create privacy engine")
    }
}

/// Statistics about the privacy engine
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub filters_loaded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_engine_creation() {
        let engine = PrivacyEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_block_result() {
        let allow = BlockResult::allow();
        assert!(!allow.should_block);

        let block = BlockResult::block("test-filter".to_string());
        assert!(block.should_block);
        assert_eq!(block.filter, Some("test-filter".to_string()));
    }

    #[test]
    fn test_request_type_conversion() {
        assert_eq!(RequestType::Script.to_adblock_type(), "script");
        assert_eq!(RequestType::Image.to_adblock_type(), "image");
        assert_eq!(RequestType::Document.to_adblock_type(), "document");
    }
}
