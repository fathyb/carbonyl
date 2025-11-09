# Chromium Integration Guide - Complete Instructions

**Status**: Ready for integration - All prerequisites complete
**Last Updated**: 2025-11-08

---

## ⚠️ Prerequisites

Before starting, ensure you have:

### Hardware Requirements
- **Disk Space**: 100GB+ free space
  - Chromium source: ~15GB
  - Build artifacts: ~50GB
  - Additional workspace: ~20GB
- **RAM**: 16GB minimum (32GB recommended)
- **CPU**: Multi-core processor (8+ cores recommended)
- **Time**: 2-4 hours for initial build

### Software Requirements
- **OS**: Linux (Ubuntu 20.04+ recommended) or macOS
- **Rust**: 1.70+ (already installed for Mongoose)
- **Python**: 3.8+
- **Git**: 2.x
- **Build tools**: gcc/clang, ninja, gn

---

## ✅ What's Already Complete

Before you start, verify these Mongoose components are ready:

```bash
# Verify Rust library builds
cd /path/to/mongoose
cargo build --lib --release
# Should complete with 0 errors

# Verify FFI tests pass
cargo test --lib ffi
# Should show: test result: ok. 3 passed

# Verify standalone test works
cd .dev-docs/examples/standalone
make run
# Should show all tests passing
```

**If all above pass**: You're ready to proceed ✅

---

## Step 1: Download Chromium Source (~30-60 minutes)

### 1.1 Install depot_tools

```bash
cd /path/to/mongoose/chromium

# depot_tools should already be present, add to PATH
export PATH="$PATH:$(pwd)/depot_tools"

# Verify
which gclient
# Should show: /path/to/mongoose/chromium/depot_tools/gclient
```

### 1.2 Sync Chromium source

```bash
cd /path/to/mongoose/chromium

# This downloads ~15GB and takes 30-60 minutes
gclient sync

# Verify download
ls -lh src/
# Should show chromium source directories
```

**Expected Result**: `chromium/src/` directory with Chromium source

---

## Step 2: Apply Mongoose Patches (~5 minutes)

### 2.1 Apply existing patches

```bash
cd /path/to/mongoose

# Apply all Mongoose patches
./scripts/patches.sh apply
```

This applies patches from:
- `chromium/patches/chromium/*.patch` - Chromium modifications
- `chromium/patches/skia/*.patch` - Skia modifications
- `chromium/patches/webrtc/*.patch` - WebRTC modifications

### 2.2 Update patches for Mongoose namespace

The patches currently use `carbonyl::` namespace. Update them:

**File**: `chromium/patches/chromium/0002-Add-Carbonyl-service.patch`

Find and replace:
```diff
- #include "carbonyl/src/browser/render_service_impl.h"
+ #include "carbonyl/src/browser/render_service_impl.h"  // No change needed

In content/browser/renderer_host/render_frame_host_impl.h:
- std::unique_ptr<carbonyl::CarbonylRenderServiceImpl> carbonyl_render_service_;
+ std::unique_ptr<mongoose::MongooseRenderServiceImpl> mongoose_render_service_;

In content/browser/renderer_host/render_frame_host_impl.cc:
- carbonyl_render_service_ = std::make_unique<carbonyl::CarbonylRenderServiceImpl>(
+ mongoose_render_service_ = std::make_unique<mongoose::MongooseRenderServiceImpl>(
      std::move(receiver));
```

**Regenerate patch**:
```bash
cd chromium/src
git add -A
git commit -m "Update to Mongoose namespace"

cd ../..
./scripts/patches.sh save
```

---

## Step 3: Generate Mojo Bindings (~10 minutes)

### 3.1 Add mongoose.mojom to build

Create `chromium/src/carbonyl/src/browser/BUILD.gn` (if not exists):

```gn
import("//mojo/public/tools/bindings/mojom.gni")

# Mojo interface for Mongoose browser service
mojom("mojom") {
  sources = [ "mongoose.mojom" ]

  public_deps = [
    "//ui/gfx/geometry/mojom",
    "//skia/public/mojom",
    "//url/mojom",
  ]

  # Generate C++ bindings
  cpp_typemaps = []
}

# Mongoose render service implementation
component("browser") {
  sources = [
    "bridge.cc",
    "bridge.h",
    "export.h",
    "host_display_client.cc",
    "host_display_client.h",
    "render_service_impl.cc",
    "render_service_impl.h",
    "renderer.cc",
    "renderer.h",
  ]

  deps = [
    ":mojom",
    "//base",
    "//mojo/public/cpp/bindings",
    "//skia",
    "//ui/gfx",
    "//url",
  ]

  # Link against Mongoose Rust library
  libs = [ "mongoose" ]
  lib_dirs = [ rebase_path("../../../../build/release") ]
}
```

### 3.2 Generate bindings

```bash
cd chromium/src

# Configure build
gn gen out/Default --args='
  is_debug=false
  is_component_build=false
  enable_nacl=false
'

# Generate Mojo bindings
autoninja -C out/Default carbonyl/src/browser:mojom

# Verify generated files
ls -lh out/Default/gen/carbonyl/src/browser/
# Should show: mongoose.mojom.h, mongoose.mojom.cc, mongoose.mojom-shared.h
```

---

## Step 4: Build Rust Library (~5 minutes)

```bash
cd /path/to/mongoose

# Build release version
cargo build --lib --release

# Verify output
ls -lh build/release/libmongoose.*
# Should show: libmongoose.so (or .dylib on macOS, .dll on Windows)

# Copy to Chromium lib directory
mkdir -p chromium/src/out/Default/lib
cp build/release/libmongoose.* chromium/src/out/Default/lib/
```

---

## Step 5: Build Chromium with Mongoose (2-4 hours)

### 5.1 Configure build

```bash
cd chromium/src

# Full browser build with Mongoose
gn gen out/Default --args='
  is_debug=false
  is_component_build=false
  symbol_level=1
  enable_nacl=false
  proprietary_codecs=true
  ffmpeg_branding="Chrome"
'
```

### 5.2 Build

```bash
# Build mongoose target (this takes 2-4 hours)
autoninja -C out/Default mongoose

# Or build everything
autoninja -C out/Default chrome
```

**Build Progress**:
- This will compile ~40,000 files
- Progress shown as: `[12345/40000] CXX obj/...`
- On 8-core CPU: ~2-3 hours
- On 16-core CPU: ~1-2 hours

### 5.3 Verify build

```bash
# Check binary exists
ls -lh out/Default/mongoose
# Should show executable

# Check Mongoose library linked
ldd out/Default/mongoose | grep mongoose
# Should show: libmongoose.so => /path/to/libmongoose.so
```

---

## Step 6: Test Integration

### 6.1 Basic startup test

```bash
cd chromium/src

# Run Mongoose browser
./out/Default/mongoose --headless --no-sandbox https://example.com

# Should see in logs:
# [INFO] Mongoose initialized with data_dir: /home/user/.config/mongoose
# [INFO] Privacy engine loaded: 12345 filters
```

### 6.2 Test privacy blocking

```bash
# Visit page with ads
./out/Default/mongoose --headless --no-sandbox https://site-with-ads.com

# Should see in logs:
# [INFO] Mongoose: Blocked https://ads.example.com/tracker.js
# [INFO] Mongoose: Blocked https://analytics.tracking.com/pixel.gif
```

### 6.3 Test ASCII rendering

```bash
# Enable ASCII mode
./out/Default/mongoose --headless --enable-ascii-mode https://example.com

# Should see in logs:
# [INFO] Hybrid ASCII mode enabled
# [INFO] Mongoose: Rendered 800x600 bitmap at (0, 0)
```

---

## Step 7: End-to-End Validation

### 7.1 Run test suite

```bash
# Run all Rust tests
cd /path/to/mongoose
cargo test --lib

# Run C++ unit tests (if created)
cd chromium/src
out/Default/mongoose_unittests

# Run integration tests
./scripts/test.sh
```

### 7.2 Manual testing checklist

- [ ] Browser starts without errors
- [ ] Privacy engine blocks known ad domains
- [ ] ASCII mode can be toggled
- [ ] Images render as ASCII art in ASCII mode
- [ ] History is persisted across sessions
- [ ] Bookmarks can be added and retrieved
- [ ] Settings are saved
- [ ] No memory leaks (run with valgrind)
- [ ] Performance meets targets (<1ms privacy checks)

---

## Troubleshooting

### Build fails with "cannot find -lmongoose"

**Solution**: Ensure Rust library is built and in the right location:
```bash
cargo build --lib --release
cp build/release/libmongoose.* chromium/src/out/Default/lib/
```

### Mojo bindings not found

**Solution**: Generate them explicitly:
```bash
cd chromium/src
autoninja -C out/Default carbonyl/src/browser:mojom
```

### Mongoose initialization fails

**Solution**: Check logs for filter download errors:
```bash
# Run with verbose logging
./out/Default/mongoose --enable-logging --v=1 https://example.com
```

### Linker errors about missing symbols

**Solution**: Verify all Rust symbols are exported:
```bash
nm -D build/release/libmongoose.so | grep mongoose
# Should show: mongoose_initialize, mongoose_check_network_request, etc.
```

---

## Performance Optimization

### Release build optimization

```bash
# In mongoose/Cargo.toml, ensure:
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### Chromium build optimization

```bash
# In chromium/src, rebuild with:
gn gen out/Release --args='
  is_debug=false
  is_official_build=true
  symbol_level=0
'
autoninja -C out/Release mongoose
```

---

## Next Steps After Integration

1. **Package for distribution**
   ```bash
   ./scripts/build.sh release
   ./scripts/copy-binaries.sh
   ```

2. **Create installer** (platform-specific)

3. **Set up CI/CD** for automated builds

4. **Performance profiling** with real workloads

5. **User testing** and feedback collection

---

## Estimated Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Environment setup | 30 min | Hardware, OS install |
| Chromium download | 30-60 min | Network speed |
| Apply patches | 10 min | - |
| Generate Mojo bindings | 10 min | - |
| Build Rust library | 5 min | - |
| **Initial Chromium build** | **2-4 hours** | CPU cores |
| Testing | 30 min | - |
| **Total (first time)** | **4-6 hours** | - |
| **Subsequent builds** | **10-30 min** | - |

---

## Support

- **Documentation**: See `.dev-docs/` directory
- **Standalone test**: `.dev-docs/examples/standalone/`
- **FFI reference**: `libmongoose/ffi_exports.h`
- **Issues**: Check build logs in `chromium/src/out/Default/`

---

**Status**: All prerequisites complete ✅
**Ready for**: Chromium download and build
**Blocker**: Requires hardware with 100GB+ disk space
