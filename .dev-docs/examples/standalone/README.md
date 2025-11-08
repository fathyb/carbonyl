# Mongoose FFI Standalone Test

This directory contains a standalone C program that tests the Mongoose FFI bridge **without requiring Chromium**. This is useful for:

- Verifying the FFI bridge works correctly
- Testing Rust library functionality independently
- Learning how to use the Mongoose C API
- Quick iteration during development

---

## Quick Start

```bash
# Build and run all tests
make run

# Or build only
make all

# Then run manually
LD_LIBRARY_PATH=../../../target/release ./test_ffi
```

---

## What Gets Tested

The standalone test suite covers:

1. **Initialization** ✅
   - `mongoose_initialize()` - Create orchestrator
   - `mongoose_is_initialized()` - Validate handle
   - `mongoose_shutdown()` - Clean shutdown

2. **Network Request Blocking** ✅
   - `mongoose_check_network_request()` - Privacy engine
   - Normal requests (should pass)
   - Advertisement requests (should block if filters loaded)
   - Filter string allocation/deallocation

3. **ASCII Mode Toggle** ✅
   - `mongoose_set_ascii_mode()` - Enable/disable rendering mode
   - Verifies FFI calls succeed

4. **Bitmap Rendering** ✅
   - `mongoose_render_bitmap()` - Render RGBA pixels as ASCII
   - Creates 8x8 test image
   - Verifies rendering pipeline works

5. **Statistics** ✅
   - `mongoose_get_stats()` - Get JSON statistics
   - `mongoose_free_string()` - Free allocated strings

6. **Error Handling** ✅
   - NULL handle rejection
   - NULL parameter rejection
   - Proper error returns

---

## Build Requirements

- **Rust toolchain** (for building libmongoose)
- **GCC or Clang** (for building C test program)
- **Standard libraries**: pthread, dl, m

On Ubuntu/Debian:
```bash
sudo apt-get install build-essential
```

On macOS:
```bash
xcode-select --install
```

---

## Expected Output

```
╔════════════════════════════════════════════════════╗
║  Mongoose FFI Bridge - Standalone Test Suite      ║
╚════════════════════════════════════════════════════╝

=== Test 1: Initialization ===
✅ PASSED: Mongoose initialized successfully
✅ PASSED: Handle validation works
✅ PASSED: Shutdown completed

=== Test 2: Network Request Blocking ===
✅ PASSED: Normal request not blocked
✅ PASSED: Advertisement blocked
   Filter: -advertisement-icon.

=== Test 3: ASCII Mode Toggle ===
✅ PASSED: ASCII mode enabled
✅ PASSED: ASCII mode disabled

=== Test 4: Bitmap Rendering ===
✅ PASSED: Bitmap rendered successfully

=== Test 5: Statistics ===
✅ PASSED: Got statistics
   Stats: {"initialized":true,"privacy_engine":"active","render_engine":"active","data_store":"active"}

=== Test 6: Error Handling ===
✅ PASSED: NULL handle rejected correctly
✅ PASSED: NULL URL rejected correctly

╔════════════════════════════════════════════════════╗
║  Test Suite Complete                               ║
╚════════════════════════════════════════════════════╝
```

---

## Notes

- **Network Dependency**: The first test may fail if it cannot download EasyList filters from the internet. This is expected in offline environments.

- **Filter Loading**: If filters fail to load, network blocking tests will show warnings but won't fail the test suite.

- **Performance**: On first run, filter list download may take 2-5 seconds. Subsequent runs use cached filters and are instant.

---

## Using in Your Own Projects

This test demonstrates the complete FFI API. To use Mongoose in your own C/C++ project:

1. **Include the header**:
   ```c
   #include "libmongoose/ffi_exports.h"
   ```

2. **Link against libmongoose**:
   ```bash
   gcc -o myapp myapp.c \
       -I/path/to/mongoose \
       -L/path/to/mongoose/target/release \
       -lmongoose \
       -lpthread -ldl -lm
   ```

3. **Use the API**:
   ```c
   MongooseHandle mongoose = mongoose_initialize("/path/to/data");

   // Check if request should be blocked
   bool should_block = false;
   mongoose_check_network_request(mongoose, url, source, type, &should_block, NULL);

   // Render bitmap
   mongoose_render_bitmap(mongoose, pixels, width, height, x, y);

   // Cleanup
   mongoose_shutdown(mongoose);
   ```

---

## Troubleshooting

**Test fails to initialize**:
- Check internet connectivity (needed for filter download)
- Check disk space in `/tmp` for cache files
- Look for Rust compilation errors in earlier steps

**Linking errors**:
- Make sure `cargo build --lib --release` completed successfully
- Verify `libmongoose.so` (or `.dylib`/`.dll`) exists in `target/release/`
- Check `LD_LIBRARY_PATH` is set correctly

**Segmentation fault**:
- Make sure you're using the correct Rust library version
- Rebuild both Rust and C components: `make clean && make all`

---

## Next Steps

After verifying the FFI works with this standalone test, you can:

1. Integrate into Chromium (see `../.dev-docs/04-CPP_Integration_Guide.md`)
2. Add your own test cases to this program
3. Use as a template for other FFI consumers

---

**Status**: Fully functional standalone test ✅
**Last Updated**: 2025-11-08
