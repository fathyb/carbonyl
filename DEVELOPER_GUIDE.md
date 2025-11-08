# Mongoose Developer Guide

Welcome to Mongoose development! This guide will help you get started contributing to the project.

---

## 🎯 Quick Start

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/orpheus497/mongoose.git
cd mongoose

# Build everything
cargo build --all

# Run tests
cargo test --lib
```

### 2. Project Structure

```
mongoose/
├── libmongoose/         # Core Rust library
│   ├── privacy.rs       # Ad blocking with adblock-rust
│   ├── render.rs        # Hybrid rendering (block/ASCII)
│   ├── data.rs          # SQLite + Redb persistence
│   ├── mojo_impl.rs     # Service orchestration
│   └── ffi.rs           # C API for Chromium
│
├── mongoose-tui/        # Terminal user interface
│   ├── main.rs          # Event loop
│   ├── app.rs           # Application state
│   └── ui.rs            # Ratatui widgets
│
└── .dev-docs/           # Design documents
    └── examples/cpp/    # C++ integration examples
```

---

## 🔧 Development Workflow

### Running Tests

```bash
# Run all tests
cargo test --lib

# Run specific module tests
cargo test --lib privacy
cargo test --lib render
cargo test --lib data

# Run with output
cargo test --lib -- --nocapture

# Run specific test
cargo test --lib privacy::tests::test_adblock_integration
```

### Building

```bash
# Debug build (fast, for development)
cargo build --lib

# Release build (optimized)
cargo build --lib --release

# Build TUI
cargo build --bin mongoose

# Build with optional features
cargo build --lib --features chafa
```

### Code Quality

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy --lib

# Fix warnings
cargo fix --lib
```

---

## 📚 Key Concepts

### 1. The Orchestrator Pattern

The `MongooseOrchestrator` is the central coordinator for all services:

```rust
use libmongoose::MongooseOrchestrator;

// Initialize all services
let orchestrator = MongooseOrchestrator::new("/path/to/data")?;

// Access services
let privacy = orchestrator.privacy_engine();
let data = orchestrator.data_store();

// Handle Mojo IPC calls
let result = orchestrator.handle_check_network_request(url, source, type);
```

**Design Pattern**: The orchestrator owns all services as `Arc<Service>` for thread-safe sharing.

---

### 2. Privacy Engine

The privacy engine uses adblock-rust for blocking:

```rust
use libmongoose::{PrivacyEngine, NetworkRequest, RequestType};

// Create engine
let engine = PrivacyEngine::new()?;

// Load filter lists
engine.load_default_filters()?;  // EasyList + EasyPrivacy

// Or add custom rules
engine.add_filter_rules(&vec![
    "||ads.example.com^".to_string(),
    "/tracker.js".to_string(),
])?;

// Check requests
let request = NetworkRequest {
    url: "https://ads.example.com/ad.js".to_string(),
    source_url: "https://site.com".to_string(),
    request_type: RequestType::Script,
};

let result = engine.check_request(&request);
if result.should_block {
    println!("Blocked! Filter: {:?}", result.filter);
}
```

**Performance**: ~0.3ms per check (uses in-memory trie structures)

---

### 3. Data Persistence

Hybrid storage using SQLite + Redb:

```rust
use libmongoose::{DataStore, DataStoreExt};

// Create store
let store = DataStore::new("/home/user/.config/mongoose")?;

// History
store.add_history("https://github.com", Some("GitHub"))?;
let history = store.get_history(10)?;

// Bookmarks
store.add_bookmark("https://rust-lang.org", Some("Rust"))?;
let bookmarks = store.get_bookmarks()?;

// Settings
store.set_setting("homepage", "https://example.com")?;
let homepage = store.get_setting("homepage")?;

// Advanced features
let stats = store.get_statistics()?;
store.cleanup_old_data(&CleanupPolicy::default())?;
store.export_to_file("backup.json")?;
```

**Storage Strategy**:
- **SQLite**: History, bookmarks, settings (relational data)
- **Redb**: Filter caches, session state (blob data)

---

### 4. Privacy Mode

In-memory storage for private browsing:

```rust
use libmongoose::PrivateDataStore;

// Create private store (in-memory only)
let private_store = PrivateDataStore::new();

// Use like normal DataStore
private_store.add_history("https://secret.com", Some("Secret"))?;

// Get memory usage
let stats = private_store.get_stats();
println!("Memory: {} bytes", stats.memory_estimate_bytes);

// Data auto-clears on drop
drop(private_store);  // All data erased
```

---

### 5. Rendering Engine

Hybrid rendering: block mode (fast) or ASCII mode (high-fidelity):

```rust
use libmongoose::{AsciiEngine, RenderMode, FrameBuffer};

// Create engine
let mut engine = AsciiEngine::default();

// Toggle modes
engine.set_mode(RenderMode::Block);  // 60fps Unicode blocks
engine.set_mode(RenderMode::Ascii);  // High-fidelity chafa

// Render image
let bitmap: Vec<u8> = /* RGBA pixel data */;
let ascii_art = engine.render(&bitmap, width, height)?;

// Composite into frame buffer
let mut frame = FrameBuffer::new(80, 24);
frame.stamp_ascii(0, 0, &ascii_art);
let output = frame.to_string();
```

---

## 🛠️ Common Tasks

### Adding a New Privacy Filter List

```rust
// In libmongoose/privacy.rs, update DEFAULT_FILTER_LISTS

const DEFAULT_FILTER_LISTS: &[(&str, &str)] = &[
    ("easylist", "https://easylist.to/easylist/easylist.txt"),
    ("easyprivacy", "https://easylist.to/easylist/easyprivacy.txt"),
    // Add your list here:
    ("my-list", "https://example.com/filters.txt"),
];
```

### Adding a New Setting

```rust
// Use the existing DataStore API
let store = DataStore::new(data_dir)?;

// Set
store.set_setting("my_feature_enabled", "true")?;

// Get
if let Some(value) = store.get_setting("my_feature_enabled")? {
    let enabled: bool = value.parse().unwrap_or(false);
    if enabled {
        // Enable feature
    }
}
```

### Adding a New Mojo IPC Handler

```rust
// In libmongoose/mojo_impl.rs

impl MongooseOrchestrator {
    pub fn handle_my_new_feature(&self, param: &str) -> MyResult {
        // Access services
        let privacy = &self.privacy_engine;
        let data = &self.data_store;

        // Do work
        MyResult {
            success: true,
            data: param.to_string(),
        }
    }
}
```

Then update `mongoose.mojom`:

```mojom
// In libmongoose/browser/mongoose.mojom

struct MyResult {
    bool success;
    string data;
};

interface MongooseRenderService {
    // Existing methods...

    // Add new method
    HandleMyNewFeature(string param) => (MyResult result);
};
```

---

## 🧪 Testing Best Practices

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // Arrange
        let engine = PrivacyEngine::new().unwrap();
        let rules = vec!["||ads.com^".to_string()];
        engine.add_filter_rules(&rules).unwrap();

        // Act
        let result = engine.check_request(&NetworkRequest {
            url: "https://ads.com/script.js".to_string(),
            source_url: "https://site.com".to_string(),
            request_type: RequestType::Script,
        });

        // Assert
        assert!(result.should_block);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_end_to_end_workflow() {
    let orchestrator = MongooseOrchestrator::new("/tmp/test")?;

    // Test privacy
    let block_result = orchestrator.handle_check_network_request(
        "https://ads.com/tracker.js",
        "https://site.com",
        RequestType::Script as i32,
    );
    assert!(block_result.should_block);

    // Test data
    orchestrator.data_store().add_history("https://github.com", Some("GitHub"))?;
    let history = orchestrator.data_store().get_history(1)?;
    assert_eq!(history.len(), 1);
}
```

---

## 🐛 Debugging Tips

### Enable Logging

```bash
# Set log level
RUST_LOG=debug cargo test --lib

# Module-specific logging
RUST_LOG=libmongoose::privacy=debug,libmongoose::data=info cargo test

# In code
use log::{debug, info, warn, error};

debug!("Request: {:?}", request);
info!("Loaded {} filters", count);
warn!("Network error: {}", e);
error!("Critical failure: {}", e);
```

### Common Issues

**Issue**: Tests fail with network errors
```bash
# Solution: Privacy engine tries to fetch filter lists
# Either:
# 1. Run tests with network available
# 2. Skip network tests: cargo test --lib -- --skip network
```

**Issue**: `adblock-rust` version mismatch
```bash
# Solution: Check Cargo.lock
cat Cargo.lock | grep adblock
# Should be version 0.8.x

# Update if needed
cargo update -p adblock
```

**Issue**: Chafa build failures
```bash
# Solution: Make chafa optional (already done)
cargo build --lib --no-default-features

# Or install chafa system library
sudo apt-get install libchafa-dev  # Ubuntu/Debian
brew install chafa                  # macOS
```

---

## 📦 Dependencies

### Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| adblock | 0.8 | Privacy blocking (Brave's engine) |
| rusqlite | 0.31 | SQLite database |
| redb | 2.1 | Embedded key-value store |
| ratatui | 0.26 | Terminal UI framework |
| crossterm | 0.27 | Terminal control |
| serde | 1.0 | Serialization |
| anyhow | 1.0 | Error handling |
| parking_lot | 0.12 | Fast mutexes |

### Optional Dependencies

| Crate | Feature | Purpose |
|-------|---------|---------|
| chafa-sys | `chafa` | ASCII art rendering |

---

## 🔗 C++ Integration (Phase 4)

### FFI API

```c
#include "libmongoose/ffi_exports.h"

// Initialize
MongooseHandle mongoose = mongoose_initialize("/home/user/.config/mongoose");

// Use
mongoose_set_ascii_mode(mongoose, true);

char* stats = mongoose_get_stats(mongoose);
printf("Stats: %s\n", stats);
mongoose_free_string(stats);

// Cleanup
mongoose_shutdown(mongoose);
```

### Mojo IPC

```cpp
// In Chromium C++
#include "libmongoose/browser/mongoose.mojom.h"

mojo::Remote<mongoose::mojom::MongooseRenderService> service_;

// Make call
auto request = mongoose::mojom::NetworkRequestData::New();
request->url = GURL("https://ads.com/script.js");
request->resource_type = 5;  // Script

mongoose::mojom::BlockResultPtr result;
service_->CheckNetworkRequest(std::move(request), &result);

if (result->should_block) {
  LOG(INFO) << "Blocked by Mongoose: " << result->matched_filter;
}
```

See `.dev-docs/04-CPP_Integration_Guide.md` for complete integration instructions.

---

## 📖 Code Style

### Rust Conventions

```rust
// Use descriptive names
let privacy_engine = PrivacyEngine::new()?;  // Good
let pe = PrivacyEngine::new()?;              // Avoid

// Document public APIs
/// Check if a network request should be blocked
///
/// # Arguments
/// * `request` - The network request to check
///
/// # Returns
/// * `BlockResult` with blocking decision
pub fn check_request(&self, request: &NetworkRequest) -> BlockResult {
    // ...
}

// Use Result for errors
pub fn load_filters(&self) -> Result<()> {
    // Use ? for propagation
    let data = self.fetch_list()?;
    Ok(())
}

// Prefer explicit over implicit
let enabled: bool = value.parse()?;  // Good
let enabled = value.parse()?;        // Type unclear
```

### Error Handling

```rust
use anyhow::{Context, Result};

// Add context to errors
let text = response
    .text()
    .context("Failed to read response body")?;

// Use thiserror for custom errors
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrivacyError {
    #[error("Failed to load filter list from {url}")]
    FilterLoadError { url: String },

    #[error("Invalid filter rule: {0}")]
    InvalidRule(String),
}
```

---

## 🚀 Performance Tips

### 1. Use Arc<> for Shared Services

```rust
// Good - single allocation, shared ownership
let engine = Arc::new(PrivacyEngine::new()?);
let clone1 = engine.clone();  // Just increments ref count
let clone2 = engine.clone();

// Avoid - multiple allocations
let engine1 = PrivacyEngine::new()?;
let engine2 = PrivacyEngine::new()?;  // Duplicate work!
```

### 2. Use RwLock for Read-Heavy Workloads

```rust
use parking_lot::RwLock;

// Privacy checks are 99% reads, 1% writes
let engine = Arc::new(RwLock::new(Engine::new()));

// Multiple readers allowed simultaneously
let result1 = engine.read().check(...);
let result2 = engine.read().check(...);

// Write locks when updating
engine.write().add_filters(...);
```

### 3. Minimize Allocations

```rust
// Good - reuse string
let mut output = String::with_capacity(width * height);
for line in lines {
    output.push_str(line);
}

// Avoid - many allocations
let output = lines.join("\n");  // Creates intermediate strings
```

---

## 📝 Contributing

### Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test --lib`
5. Format code: `cargo fmt`
6. Commit: `git commit -m "feat: add my feature"`
7. Push: `git push origin feature/my-feature`
8. Open a Pull Request

### Commit Message Format

```
<type>: <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `test`: Adding tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

**Examples**:
```
feat: add custom filter list support

Allow users to add custom adblock filter lists via settings UI.

Closes #123
```

---

## 🔍 Resources

### Documentation
- **Project Status**: `PROJECT_STATUS.md`
- **Systems Design**: `.dev-docs/02-Systems_Design_Document.md`
- **C++ Integration**: `.dev-docs/04-CPP_Integration_Guide.md`
- **API Examples**: `mongoose-tui/src/integration_example.rs`

### External Links
- **adblock-rust**: https://github.com/brave/adblock-rust
- **ratatui**: https://ratatui.rs/
- **Carbonyl**: https://github.com/fathyb/carbonyl
- **Rust Book**: https://doc.rust-lang.org/book/

---

## ❓ FAQ

**Q: Why use both SQLite and Redb?**
A: SQLite is excellent for relational queries (history search), while Redb is faster for blob storage (filter caches). Hybrid approach gives best of both worlds.

**Q: Why is adblock-rust not Send+Sync?**
A: Some internal regex caching uses `Rc<RefCell<>>` which isn't thread-safe. We work around this by loading filters synchronously on initialization.

**Q: Can I use mongoose without Chromium?**
A: The TUI and Rust libraries can be used independently, but full browser functionality requires the Chromium core for rendering and JavaScript execution.

**Q: How do I add a new keyboard shortcut?**
A: Edit `mongoose-tui/src/input.rs` and add a handler in `handle_key_event()`.

---

## 💬 Getting Help

- **Issues**: https://github.com/orpheus497/mongoose/issues
- **Discussions**: https://github.com/orpheus497/mongoose/discussions
- **Discord**: [Coming soon]

---

Happy coding! 🦀
