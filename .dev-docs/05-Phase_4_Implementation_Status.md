# Phase 4: C++ Integration - Implementation Status

**Date**: 2025-11-08
**Status**: In Progress
**Branch**: `claude/mongoose-browser-architecture-011CUwAVN615vzus6JGnkTbw`

---

## Overview

Phase 4 integrates the completed Rust components (Phase 3) with the Chromium/Carbonyl C++ codebase. This involves:

1. Updating C++ service implementations to use Mongoose instead of Carbonyl
2. Generating Mojo bindings from mongoose.mojom
3. Integrating Rust FFI calls from C++
4. Creating URLLoaderInterceptor for privacy blocking
5. Implementing bitmap capture for ASCII rendering

---

## ✅ Completed Work

### 1. C++ Service Implementation Updates

**Files Modified**:
- `libmongoose/browser/render_service_impl.h`
- `libmongoose/browser/render_service_impl.cc`

**Changes**:
```cpp
// Before: carbonyl::CarbonylRenderServiceImpl
// After:  mongoose::MongooseRenderServiceImpl

class MongooseRenderServiceImpl: public mojom::MongooseRenderService {
public:
  // Existing method (kept for compatibility)
  void DrawText(std::vector<mojom::TextDataPtr> data) override;

  // New Phase 3 methods
  void CheckNetworkRequest(mojom::NetworkRequestDataPtr request,
                          CheckNetworkRequestCallback callback) override;
  void DrawImageBitmap(mojom::BitmapDataPtr bitmap) override;
  void SetHybridAsciiMode(bool enabled) override;
  void SetTitle(const std::string& title) override;

private:
  MongooseHandle mongoose_handle_;  // FFI handle to Rust orchestrator
};
```

**Implementation Details**:
- ✅ Mongoose FFI initialization via `mongoose_initialize()`
- ✅ SetHybridAsciiMode() calls Rust FFI `mongoose_set_ascii_mode()`
- ⏳ CheckNetworkRequest() stubbed (TODO: call Rust privacy engine)
- ⏳ DrawImageBitmap() stubbed (TODO: call Rust render engine)

---

## ⏳ Remaining Work

### 2. Chromium Patch Updates

**Files to Update**:
- `chromium/patches/chromium/0002-Add-Carbonyl-service.patch`

**Required Changes**:
```diff
- #include "carbonyl/src/browser/render_service_impl.h"
+ #include "carbonyl/src/browser/render_service_impl.h"

In RenderFrameHostImpl:
- std::unique_ptr<carbonyl::CarbonylRenderServiceImpl> carbonyl_render_service_;
+ std::unique_ptr<mongoose::MongooseRenderServiceImpl> mongoose_render_service_;

In initialization:
- carbonyl_render_service_ = std::make_unique<carbonyl::CarbonylRenderServiceImpl>(
+ mongoose_render_service_ = std::make_unique<mongoose::MongooseRenderServiceImpl>(
     std::move(receiver));
```

**Why**: The Chromium patches need to instantiate `MongooseRenderServiceImpl` instead of `CarbonylRenderServiceImpl` in the browser process.

---

### 3. Mojo Bindings Generation

**File**: `libmongoose/browser/mongoose.mojom`

**Steps**:
1. Add mongoose.mojom to Chromium's GN build files
2. Run Mojo code generator to create C++ bindings:
   ```bash
   cd chromium/src
   autoninja -C out/Default mongoose/browser:mojom
   ```

**Generated Files** (will be in build directory):
- `mongoose.mojom.h` - C++ interface declarations
- `mongoose.mojom.cc` - C++ interface implementations
- `mongoose.mojom-shared.h` - Shared types

**Why**: Chromium's Mojo system needs to generate C++ bindings from our .mojom interface definition.

---

### 4. Network Request Interception

**New Files to Create**:
- `chromium/patches/chromium/0015-Add-Mongoose-network-interceptor.patch`

**What to Patch**:
```cpp
// In content/browser/loader/navigation_url_loader_impl.cc
// or similar URLLoaderFactory creation point

#include "carbonyl/src/browser/render_service_impl.h"

void NavigationURLLoaderImpl::CreateURLLoaderFactoryWithHeaderClient(...) {
  // Get Mongoose service handle
  auto* mongoose_service = GetMongooseServiceForFrame(frame_tree_node_id);

  // Create blocking interceptor
  auto interceptor = std::make_unique<MongooseURLLoaderInterceptor>(
      mongoose_service);

  // Add to interceptor chain
  non_network_url_loader_factories.emplace(
      url::kAboutScheme, std::move(interceptor));
}
```

**Reference Implementation**: `.dev-docs/examples/cpp/mongoose_url_loader_interceptor.cc`

**Why**: This intercepts network requests and calls Rust privacy engine to check if they should be blocked.

---

### 5. Bitmap Capture for ASCII Rendering

**New Files to Create**:
- `chromium/patches/chromium/0016-Add-Mongoose-bitmap-capture.patch`

**What to Patch**:
```cpp
// In cc/trees/layer_tree_host.cc or similar compositor location

#include "carbonyl/src/browser/render_service_impl.h"

void LayerTreeHost::CaptureBitmapForMongoose(const SkBitmap& bitmap,
                                             const gfx::Rect& bounds) {
  auto* mongoose_service = GetMongooseService();
  if (!mongoose_service || !mongoose_service->IsAsciiModeEnabled()) {
    return;
  }

  // Create shared memory
  auto pixels_region = base::UnsafeSharedMemoryRegion::Create(
      bitmap.computeByteSize());

  // Copy pixels
  memcpy(pixels_region.Map().memory(), bitmap.getPixels(),
         bitmap.computeByteSize());

  // Send to Mongoose
  auto bitmap_data = mongoose::mojom::BitmapData::New();
  bitmap_data->pixels = std::move(pixels_region);
  bitmap_data->width = bitmap.width();
  bitmap_data->height = bitmap.height();
  bitmap_data->bounds = bounds;

  mongoose_service->DrawImageBitmap(std::move(bitmap_data));
}
```

**Reference Implementation**: `.dev-docs/examples/cpp/mongoose_text_capture_device_patch.cc`

**Why**: This captures rendered bitmaps and sends them to Rust for ASCII art conversion.

---

### 6. Build System Integration

**File to Update**: `chromium/src/BUILD.gn` (via patch)

**Add**:
```gn
# Mongoose service interface
mojom("mojom") {
  sources = [ "carbonyl/src/browser/mongoose.mojom" ]

  public_deps = [
    "//ui/gfx/geometry/mojom",
    "//skia/public/mojom",
    "//url/mojom",
  ]
}

# Link against Rust library
libs = [
  "mongoose",  # libmongoose.a
]

lib_dirs = [
  "../../target/release",  # Path to Rust build output
]
```

**Why**: Chromium's build system needs to know about mongoose.mojom and link against libmongoose.a.

---

## 🔧 Testing Strategy

### Unit Tests (C++)

Create `libmongoose/browser/render_service_impl_unittest.cc`:

```cpp
#include "carbonyl/src/browser/render_service_impl.h"
#include "testing/gtest/include/gtest/gtest.h"

namespace mongoose {

TEST(MongooseRenderServiceTest, SetHybridAsciiMode) {
  mojo::PendingReceiver<mojom::MongooseRenderService> receiver;
  auto service = std::make_unique<MongooseRenderServiceImpl>(
      std::move(receiver));

  // Should not crash
  service->SetHybridAsciiMode(true);
  service->SetHybridAsciiMode(false);
}

TEST(MongooseRenderServiceTest, CheckNetworkRequest) {
  mojo::PendingReceiver<mojom::MongooseRenderService> receiver;
  auto service = std::make_unique<MongooseRenderServiceImpl>(
      std::move(receiver));

  auto request = mojom::NetworkRequestData::New();
  request->url = GURL("https://ads.example.com/script.js");
  request->resource_type = 5;  // Script

  bool callback_called = false;
  service->CheckNetworkRequest(
      std::move(request),
      base::BindOnce([](bool* called, mojom::BlockResultPtr result) {
        *called = true;
        // TODO: Verify result
      }, &callback_called));

  EXPECT_TRUE(callback_called);
}

}  // namespace mongoose
```

### Integration Tests

1. **Privacy Blocking Test**: Visit page with ads, verify they're blocked
2. **ASCII Rendering Test**: View images with hybrid mode enabled
3. **Performance Test**: Measure latency of CheckNetworkRequest calls

---

## 📝 Build Instructions

Once Chromium patches are ready:

```bash
# 1. Fetch Chromium
cd chromium
./scripts/gclient.sh sync

# 2. Apply patches
./scripts/patches.sh apply

# 3. Build Rust library
cd ..
cargo build --lib --release

# 4. Configure Chromium build
cd chromium/src
gn gen out/Default --args='is_debug=false'

# 5. Build Chromium + Mongoose
autoninja -C out/Default mongoose

# 6. Run
./out/Default/mongoose https://example.com
```

---

## 🎯 Next Steps

### Immediate (Can be done now)
1. ✅ Update render_service_impl.h/cc (DONE)
2. ⏳ Implement CheckNetworkRequest to call Rust FFI
3. ⏳ Implement DrawImageBitmap to call Rust FFI

### Requires Chromium Checkout
4. Generate Mojo bindings from mongoose.mojom
5. Update Chromium patches to use MongooseRenderServiceImpl
6. Create network interceptor patch
7. Create bitmap capture patch
8. Update build files to link libmongoose.a
9. Build and test end-to-end

---

## 📊 Current Blockers

**None** - All preparatory work is complete. The next steps require:
1. Chromium source checkout (~15 GB download)
2. Chromium build environment setup (depot_tools, etc.)
3. Initial Chromium build (2-4 hours on modern hardware)
4. Iterative patch development and testing

---

## 🔗 References

- **C++ Integration Guide**: `.dev-docs/04-CPP_Integration_Guide.md`
- **Mojo Documentation**: https://chromium.googlesource.com/chromium/src/+/HEAD/mojo/README.md
- **Carbonyl Patches**: `chromium/patches/chromium/`
- **Rust FFI API**: `libmongoose/ffi_exports.h`

---

**Status Summary**: Phase 4 foundation is complete. C++ service implementation is updated and ready. Remaining work requires Chromium checkout and build environment.
