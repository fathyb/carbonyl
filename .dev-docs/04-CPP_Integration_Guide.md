# C++ Integration Guide for Mongoose Browser

## Overview

This document provides detailed instructions for integrating the Rust libmongoose components with the Chromium C++ core. This work is required to complete Phase 3 and Phase 4 of the mongoose implementation.

**Status**: Rust components are complete and tested. C++ integration work is pending.

## Prerequisites

- Chromium build environment set up (see Carbonyl documentation)
- Mongoose Mojo interface defined in `libmongoose/browser/mongoose.mojom`
- Rust libmongoose library compiled successfully

## Integration Architecture

```
┌─────────────────┐
│  mongoose-tui   │ (Ratatui UI - Rust)
│   (Terminal)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  libmongoose    │ (Privacy, Data, Rendering - Rust)
│  (Rust Library) │
└────────┬────────┘
         │ Mojo IPC
         ▼
┌─────────────────┐
│ mongoose-core   │ (Chromium fork - C++)
│   (Browser)     │
└─────────────────┘
```

## Required C++ Modifications

### 1. Network Request Interceptor (Privacy Engine Integration)

**File**: `mongoose-core/content/browser/mongoose_url_loader_interceptor.h`

**Purpose**: Intercept all network requests and call the Rust privacy engine to check for blocking rules.

**Implementation**:

```cpp
#ifndef MONGOOSE_CORE_CONTENT_BROWSER_MONGOOSE_URL_LOADER_INTERCEPTOR_H_
#define MONGOOSE_CORE_CONTENT_BROWSER_MONGOOSE_URL_LOADER_INTERCEPTOR_H_

#include "content/public/browser/url_loader_request_interceptor.h"
#include "third_party/blink/public/mojom/loader/resource_load_info.mojom-shared.h"
#include "libmongoose/browser/mongoose.mojom.h"

namespace content {

class MongooseURLLoaderInterceptor : public URLLoaderRequestInterceptor {
 public:
  MongooseURLLoaderInterceptor();
  ~MongooseURLLoaderInterceptor() override;

  // URLLoaderRequestInterceptor implementation
  void MaybeCreateLoader(
      const network::ResourceRequest& tentative_resource_request,
      BrowserContext* browser_context,
      LoaderCallback callback,
      FallbackCallback fallback_callback) override;

 private:
  // Mojo interface to Rust privacy engine
  mojo::Remote<mongoose::mojom::MongooseRenderService> mongoose_service_;
};

}  // namespace content

#endif
```

**File**: `mongoose-core/content/browser/mongoose_url_loader_interceptor.cc`

```cpp
#include "content/browser/mongoose_url_loader_interceptor.h"
#include "content/public/browser/browser_thread.h"
#include "services/network/public/cpp/resource_request.h"
#include "net/base/net_errors.h"

namespace content {

MongooseURLLoaderInterceptor::MongooseURLLoaderInterceptor() {
  // TODO: Connect to Rust MongooseOrchestrator via Mojo
  // This will be initialized from the browser process
}

MongooseURLLoaderInterceptor::~MongooseURLLoaderInterceptor() = default;

void MongooseURLLoaderInterceptor::MaybeCreateLoader(
    const network::ResourceRequest& tentative_resource_request,
    BrowserContext* browser_context,
    LoaderCallback callback,
    FallbackCallback fallback_callback) {

  DCHECK_CURRENTLY_ON(BrowserThread::UI);

  // Create NetworkRequestData for Mojo call
  auto request_data = mongoose::mojom::NetworkRequestData::New();
  request_data->url = tentative_resource_request.url;
  request_data->initiator = tentative_resource_request.request_initiator;
  request_data->resource_type = static_cast<int32_t>(
      tentative_resource_request.resource_type);

  // Synchronous Mojo call to Rust privacy engine
  // IMPORTANT: This must be fast (microseconds) to avoid blocking the browser
  mongoose::mojom::BlockResultPtr result;
  bool success = mongoose_service_->CheckNetworkRequest(
      std::move(request_data), &result);

  if (success && result->should_block) {
    // Block the request
    LOG(INFO) << "Mongoose: Blocking request to "
              << tentative_resource_request.url.spec()
              << " (filter: " << result->matched_filter.value_or("unknown") << ")";

    // Return error to abort the request
    std::move(callback).Run(
        base::MakeRefCounted<network::SingleRequestURLLoaderFactory>(
            base::BindOnce([](const network::ResourceRequest& request,
                            mojo::PendingReceiver<network::mojom::URLLoader> receiver,
                            mojo::PendingRemote<network::mojom::URLLoaderClient> client) {
              mojo::Remote<network::mojom::URLLoaderClient> client_remote(std::move(client));
              client_remote->OnComplete(network::URLLoaderCompletionStatus(net::ERR_BLOCKED_BY_CLIENT));
            })));
    return;
  }

  // Allow the request - call fallback to use default loader
  std::move(fallback_callback).Run();
}

}  // namespace content
```

**Integration Point**: Register the interceptor in `ContentBrowserClient`:

```cpp
// In mongoose-core/content/browser/content_browser_client.cc

std::vector<std::unique_ptr<URLLoaderRequestInterceptor>>
MongooseContentBrowserClient::WillCreateURLLoaderRequestInterceptors(
    NavigationUIData* navigation_ui_data,
    int frame_tree_node_id) {
  std::vector<std::unique_ptr<URLLoaderRequestInterceptor>> interceptors;

  // Add Mongoose privacy interceptor
  interceptors.push_back(std::make_unique<MongooseURLLoaderInterceptor>());

  return interceptors;
}
```

---

### 2. Bitmap Capture for ASCII Art Rendering

**File**: `mongoose-core/ui/gfx/mongoose_text_capture_device.h` (modify existing)

**Purpose**: Capture raw bitmap data from Skia canvas and send to Rust rendering engine for ASCII art conversion.

**Modifications**:

```cpp
// Add to existing MongooseTextCaptureDevice class

class MongooseTextCaptureDevice : public SkCanvas {
 public:
  // ... existing methods ...

  // NEW: Global flag for hybrid ASCII mode
  static void SetHybridAsciiMode(bool enabled);
  static bool IsHybridAsciiModeEnabled();

 protected:
  // Override drawBitmap to intercept image rendering
  void onDrawBitmap(const SkBitmap& bitmap,
                   SkScalar left,
                   SkScalar top,
                   const SkPaint* paint) override;

 private:
  // NEW: Send bitmap to Rust for ASCII conversion
  void SendBitmapToMongoose(const SkBitmap& bitmap,
                           const gfx::Rect& bounds);

  // NEW: Global state
  static bool g_hybrid_ascii_mode_;

  // NEW: Mojo interface to Rust rendering engine
  mojo::Remote<mongoose::mojom::MongooseRenderService> mongoose_service_;
};
```

**File**: `mongoose-core/ui/gfx/mongoose_text_capture_device.cc` (modify existing)

```cpp
// Static member initialization
bool MongooseTextCaptureDevice::g_hybrid_ascii_mode_ = false;

void MongooseTextCaptureDevice::SetHybridAsciiMode(bool enabled) {
  g_hybrid_ascii_mode_ = enabled;
  LOG(INFO) << "Mongoose: ASCII art mode " << (enabled ? "enabled" : "disabled");
}

bool MongooseTextCaptureDevice::IsHybridAsciiModeEnabled() {
  return g_hybrid_ascii_mode_;
}

void MongooseTextCaptureDevice::onDrawBitmap(const SkBitmap& bitmap,
                                             SkScalar left,
                                             SkScalar top,
                                             const SkPaint* paint) {
  if (g_hybrid_ascii_mode_) {
    // Send to Rust for high-fidelity ASCII rendering
    gfx::Rect bounds(left, top, bitmap.width(), bitmap.height());
    SendBitmapToMongoose(bitmap, bounds);
  } else {
    // Use fast block rendering (existing Carbonyl logic)
    // ... existing code ...
  }

  // Call parent implementation
  SkCanvas::onDrawBitmap(bitmap, left, top, paint);
}

void MongooseTextCaptureDevice::SendBitmapToMongoose(
    const SkBitmap& bitmap,
    const gfx::Rect& bounds) {

  // Convert SkBitmap to RGBA byte array
  SkImageInfo info = SkImageInfo::MakeN32Premul(bitmap.width(), bitmap.height());
  size_t row_bytes = info.minRowBytes();
  size_t pixel_data_size = info.computeByteSize(row_bytes);

  std::vector<uint8_t> pixel_data(pixel_data_size);
  bitmap.readPixels(info, pixel_data.data(), row_bytes, 0, 0);

  // Create shared memory region for pixel data
  mojo::ScopedSharedBufferHandle shared_buffer =
      mojo::SharedBufferHandle::Create(pixel_data_size);

  mojo::ScopedSharedBufferMapping mapping = shared_buffer->Map(pixel_data_size);
  memcpy(mapping.get(), pixel_data.data(), pixel_data_size);

  // Create BitmapData for Mojo call
  auto bitmap_data = mongoose::mojom::BitmapData::New();
  bitmap_data->pixels = std::move(shared_buffer);
  bitmap_data->width = bitmap.width();
  bitmap_data->height = bitmap.height();
  bitmap_data->bounds = bounds;

  // Send to Rust (async call - fire and forget)
  mongoose_service_->DrawImageBitmap(std::move(bitmap_data));
}
```

---

### 3. Mojo Bindings Generation

**Required**: Generate C++ and Rust bindings from `mongoose.mojom`

**Command** (run from Chromium build directory):

```bash
# Generate C++ bindings
$CHROMIUM_SRC/mojo/public/tools/bindings/mojom_bindings_generator.py \
  --output_dir=out/Default/gen \
  --bytecode_path=... \
  libmongoose/browser/mongoose.mojom

# Generate Rust bindings
$CHROMIUM_SRC/mojo/public/tools/bindings/mojom_bindings_generator.py \
  --generate_rust_bindings \
  --output_dir=libmongoose/browser/generated \
  libmongoose/browser/mongoose.mojom
```

**Files Generated**:
- `out/Default/gen/libmongoose/browser/mongoose.mojom.h` (C++)
- `out/Default/gen/libmongoose/browser/mongoose.mojom.cc` (C++)
- `libmongoose/browser/generated/mongoose.mojom.rs` (Rust)

**Update Rust `lib.rs`** to include generated bindings:

```rust
// In libmongoose/lib.rs
#[allow(non_snake_case, non_camel_case_types)]
mod mongoose_mojo {
    include!(concat!(env!("OUT_DIR"), "/mongoose.mojom.rs"));
}

pub use mongoose_mojo::*;
```

---

### 4. Rust-C++ Bridge Setup

**File**: `libmongoose/ffi.rs` (new file)

**Purpose**: Provide C-compatible FFI functions for C++ to call into Rust.

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::mojo_impl::MongooseOrchestrator;
use std::sync::Mutex;

// Global orchestrator instance
static ORCHESTRATOR: Mutex<Option<MongooseOrchestrator>> = Mutex::new(None);

/// Initialize the mongoose orchestrator
/// Called from C++ during browser startup
#[no_mangle]
pub extern "C" fn mongoose_initialize(data_dir: *const c_char) -> bool {
    let data_dir_str = unsafe {
        if data_dir.is_null() {
            return false;
        }
        CStr::from_ptr(data_dir).to_str().unwrap_or("")
    };

    match MongooseOrchestrator::new(data_dir_str) {
        Ok(orchestrator) => {
            *ORCHESTRATOR.lock().unwrap() = Some(orchestrator);
            true
        }
        Err(e) => {
            eprintln!("Failed to initialize mongoose: {}", e);
            false
        }
    }
}

/// Shutdown the mongoose orchestrator
#[no_mangle]
pub extern "C" fn mongoose_shutdown() {
    *ORCHESTRATOR.lock().unwrap() = None;
}

/// Get access to the orchestrator
fn with_orchestrator<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&MongooseOrchestrator) -> R,
{
    ORCHESTRATOR.lock().unwrap().as_ref().map(f)
}
```

**File**: `libmongoose/ffi_exports.h` (new file)

**Purpose**: C header for C++ code to include.

```cpp
#ifndef LIBMONGOOSE_FFI_EXPORTS_H_
#define LIBMONGOOSE_FFI_EXPORTS_H_

#ifdef __cplusplus
extern "C" {
#endif

// Initialize mongoose with data directory path
bool mongoose_initialize(const char* data_dir);

// Shutdown mongoose
void mongoose_shutdown();

#ifdef __cplusplus
}
#endif

#endif  // LIBMONGOOSE_FFI_EXPORTS_H_
```

---

## Build System Integration

### Update `Cargo.toml` to build C-compatible library:

```toml
[lib]
name = "mongoose"
path = "libmongoose/lib.rs"
crate-type = ["cdylib", "staticlib"]  # Add staticlib for linking
```

### Update Chromium GN build file:

```python
# In mongoose-core/BUILD.gn

shared_library("mongoose") {
  sources = [
    "content/browser/mongoose_url_loader_interceptor.cc",
    "content/browser/mongoose_url_loader_interceptor.h",
    # ... other mongoose sources ...
  ]

  deps = [
    "//libmongoose:mongoose_rust",  # Rust library dependency
    "//mojo/public/cpp/bindings",
    "//content/public/browser",
  ]
}

# Rust library target
rust_static_library("mongoose_rust") {
  crate_root = "//libmongoose/lib.rs"
  sources = [ "//libmongoose/lib.rs" ]

  deps = [
    # Rust dependencies managed by Cargo
  ]
}
```

---

## Testing the Integration

### 1. Test Privacy Blocking

```cpp
// In Chromium C++ test file
TEST_F(MongooseTest, PrivacyEngineBlocks) {
  // Initialize mongoose
  ASSERT_TRUE(mongoose_initialize("/tmp/mongoose-test"));

  // Navigate to a page with ads
  NavigateToURL("https://example.com/page-with-ads");

  // Verify ad requests were blocked
  // Check network log for ERR_BLOCKED_BY_CLIENT errors
}
```

### 2. Test ASCII Rendering

```cpp
TEST_F(MongooseTest, AsciiRenderingMode) {
  ASSERT_TRUE(mongoose_initialize("/tmp/mongoose-test"));

  // Enable ASCII mode
  MongooseTextCaptureDevice::SetHybridAsciiMode(true);

  // Navigate to page with images
  NavigateToURL("https://example.com/gallery");

  // Verify bitmaps were sent to Rust
  // Check that DrawImageBitmap Mojo calls were made
}
```

---

## Performance Considerations

### Privacy Engine

- **Synchronous blocking check**: Must complete in microseconds
- **Solution**: adblock-rust uses in-memory trie structures for O(log n) lookups
- **Benchmark**: Aim for <1ms per request check

### ASCII Rendering

- **Bitmap transfer**: Use shared memory to avoid copying
- **Chafa processing**: Run in separate thread to avoid blocking UI
- **Framebuffer compositing**: Batch updates to minimize IPC calls

---

## Debugging Tips

### Enable Mojo Tracing

```bash
chrome --enable-logging --vmodule=mojo=2
```

### Check Rust Logs

```bash
RUST_LOG=debug chrome
```

### Monitor Blocking

```bash
# In C++ code
VLOG(1) << "Mongoose blocked: " << url;

# In Rust code
debug!("Blocking request: {}", url);
```

---

## Next Steps

1. **Generate Mojo bindings** for mongoose.mojom
2. **Implement URL interceptor** in C++
3. **Implement bitmap capture** in MongooseTextCaptureDevice
4. **Test integration** with real filter lists
5. **Performance profiling** to ensure <1ms blocking checks
6. **End-to-end testing** with mongoose-tui

---

## References

- [Carbonyl Architecture](https://github.com/fathyb/carbonyl)
- [Chromium Content API](https://chromium.googlesource.com/chromium/src/+/main/content/public/)
- [Mojo IPC Documentation](https://chromium.googlesource.com/chromium/src/+/main/mojo/README.md)
- [adblock-rust Repository](https://github.com/brave/adblock-rust)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)

---

**Author**: orpheus497 (design), AI Implementation Team
**Last Updated**: 2025-11-08
**Status**: Rust components complete ✅ | C++ integration pending ⏳
