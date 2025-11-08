/// Privacy Engine - Ad and tracker blocking using adblock-rust
///
/// This module integrates the adblock-rust engine (the same engine used by Brave)
/// to provide network-level blocking of ads and trackers.

use adblock::{
    lists::{FilterSet, ParseOptions},
    request::Request,
    Engine,
};
use anyhow::{Context, Result};
use log::{debug, info, warn};
use parking_lot::RwLock;
use reqwest::blocking::Client;
use std::sync::Arc;
use std::time::Duration;

/// Default filter lists to load on first run
const DEFAULT_FILTER_LISTS: &[(&str, &str)] = &[
    ("easylist", "https://easylist.to/easylist/easylist.txt"),
    (
        "easyprivacy",
        "https://easylist.to/easylist/easyprivacy.txt",
    ),
];

/// How long before we consider a filter list stale (24 hours)
/// This will be used in the future for automatic filter list updates
#[allow(dead_code)]
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
        // Create a FilterSet and add all rules
        let debug_info = cfg!(debug_assertions);
        let mut filter_set = FilterSet::new(debug_info);
        filter_set.add_filters(rules, ParseOptions::default());

        // Create engine from the filter set (true = optimize)
        let new_engine = Engine::from_filter_set(filter_set, true);

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
        let engine = self.engine.read();

        // Create an adblock Request object
        let adblock_request = match Request::new(
            &request.url,
            &request.source_url,
            request.request_type.to_adblock_type(),
        ) {
            Ok(req) => req,
            Err(e) => {
                warn!("Failed to create adblock request: {}", e);
                return BlockResult::allow();
            }
        };

        // Check if the request should be blocked
        let blocker_result = engine.check_network_request(&adblock_request);

        if blocker_result.matched {
            debug!(
                "Blocking request: {} (matched filter: {:?})",
                request.url, blocker_result.filter
            );

            BlockResult::block(
                blocker_result
                    .filter
                    .unwrap_or_else(|| "unknown".to_string()),
            )
        } else {
            BlockResult::allow()
        }
    }

    /// Get statistics about the loaded filters
    pub fn get_stats(&self) -> EngineStats {
        let _engine = self.engine.read();

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

    #[test]
    fn test_adblock_integration() {
        let engine = PrivacyEngine::new().unwrap();

        // Add some test filter rules
        let rules = vec![
            "-advertisement-icon.".to_string(),
            "-advertisement.".to_string(),
            "||ads.example.com^".to_string(),
        ];

        engine.add_filter_rules(&rules).unwrap();

        // Test that ad URLs are blocked
        let ad_request = NetworkRequest {
            url: "http://example.com/-advertisement-icon.png".to_string(),
            source_url: "http://example.com/".to_string(),
            request_type: RequestType::Image,
        };

        let result = engine.check_request(&ad_request);
        assert!(result.should_block, "Advertisement should be blocked");

        // Test that normal URLs are allowed
        let normal_request = NetworkRequest {
            url: "http://example.com/normal-image.png".to_string(),
            source_url: "http://example.com/".to_string(),
            request_type: RequestType::Image,
        };

        let result = engine.check_request(&normal_request);
        assert!(!result.should_block, "Normal request should be allowed");
    }
}
