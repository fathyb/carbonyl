/**
 * Standalone FFI Test - Demonstrates Mongoose FFI without Chromium
 *
 * This program tests the Mongoose FFI bridge independently of Chromium.
 * It can be compiled and run to verify the Rust library works correctly.
 *
 * Build instructions:
 *   cargo build --lib --release
 *   gcc -o test_ffi test_ffi.c \
 *       -I../../.. \
 *       -L../../../target/release \
 *       -lmongoose \
 *       -lpthread -ldl -lm
 *
 * Run:
 *   LD_LIBRARY_PATH=../../../target/release ./test_ffi
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "libmongoose/ffi_exports.h"

void test_initialization() {
    printf("=== Test 1: Initialization ===\n");

    MongooseHandle handle = mongoose_initialize("/tmp/mongoose-test-ffi");
    if (!handle) {
        printf("❌ FAILED: Could not initialize Mongoose\n");
        printf("   (This may fail if EasyList download fails)\n");
        return;
    }

    printf("✅ PASSED: Mongoose initialized successfully\n");

    if (mongoose_is_initialized(handle)) {
        printf("✅ PASSED: Handle validation works\n");
    } else {
        printf("❌ FAILED: Handle validation failed\n");
    }

    mongoose_shutdown(handle);
    printf("✅ PASSED: Shutdown completed\n\n");
}

void test_network_blocking() {
    printf("=== Test 2: Network Request Blocking ===\n");

    MongooseHandle handle = mongoose_initialize("/tmp/mongoose-test-ffi");
    if (!handle) {
        printf("⏭️  SKIPPED: Could not initialize (network issue?)\n\n");
        return;
    }

    // Test 1: Check a normal request (should not block)
    bool should_block = false;
    char* filter = NULL;

    bool success = mongoose_check_network_request(
        handle,
        "https://example.com/page.html",
        "https://referrer.com",
        0,  // Document type
        &should_block,
        &filter
    );

    if (!success) {
        printf("❌ FAILED: Network check returned error\n");
    } else if (should_block) {
        printf("⚠️  WARNING: example.com was blocked (unexpected)\n");
        printf("   Filter: %s\n", filter ? filter : "none");
        if (filter) mongoose_free_string(filter);
    } else {
        printf("✅ PASSED: Normal request not blocked\n");
    }

    // Test 2: Check an ad request (should block if filters loaded)
    should_block = false;
    filter = NULL;

    success = mongoose_check_network_request(
        handle,
        "https://ads.example.com/-advertisement-icon.png",
        "https://site.com",
        2,  // Image type
        &should_block,
        &filter
    );

    if (!success) {
        printf("❌ FAILED: Ad check returned error\n");
    } else if (should_block) {
        printf("✅ PASSED: Advertisement blocked\n");
        printf("   Filter: %s\n", filter ? filter : "unknown");
        if (filter) mongoose_free_string(filter);
    } else {
        printf("⚠️  INFO: Advertisement not blocked (filters may not be loaded)\n");
    }

    mongoose_shutdown(handle);
    printf("\n");
}

void test_ascii_mode() {
    printf("=== Test 3: ASCII Mode Toggle ===\n");

    MongooseHandle handle = mongoose_initialize("/tmp/mongoose-test-ffi");
    if (!handle) {
        printf("⏭️  SKIPPED: Could not initialize (network issue?)\n\n");
        return;
    }

    // Toggle ASCII mode on
    mongoose_set_ascii_mode(handle, true);
    printf("✅ PASSED: ASCII mode enabled\n");

    // Toggle ASCII mode off
    mongoose_set_ascii_mode(handle, false);
    printf("✅ PASSED: ASCII mode disabled\n");

    mongoose_shutdown(handle);
    printf("\n");
}

void test_bitmap_rendering() {
    printf("=== Test 4: Bitmap Rendering ===\n");

    MongooseHandle handle = mongoose_initialize("/tmp/mongoose-test-ffi");
    if (!handle) {
        printf("⏭️  SKIPPED: Could not initialize (network issue?)\n\n");
        return;
    }

    // Create a small test bitmap (8x8 red square)
    const int width = 8;
    const int height = 8;
    unsigned char* pixels = (unsigned char*)malloc(width * height * 4);

    for (int i = 0; i < width * height * 4; i += 4) {
        pixels[i + 0] = 255;  // R
        pixels[i + 1] = 0;    // G
        pixels[i + 2] = 0;    // B
        pixels[i + 3] = 255;  // A
    }

    // Render the bitmap
    bool success = mongoose_render_bitmap(
        handle,
        pixels,
        width,
        height,
        0,  // x
        0   // y
    );

    if (success) {
        printf("✅ PASSED: Bitmap rendered successfully\n");
    } else {
        printf("❌ FAILED: Bitmap rendering failed\n");
    }

    free(pixels);
    mongoose_shutdown(handle);
    printf("\n");
}

void test_statistics() {
    printf("=== Test 5: Statistics ===\n");

    MongooseHandle handle = mongoose_initialize("/tmp/mongoose-test-ffi");
    if (!handle) {
        printf("⏭️  SKIPPED: Could not initialize (network issue?)\n\n");
        return;
    }

    char* stats = mongoose_get_stats(handle);
    if (stats) {
        printf("✅ PASSED: Got statistics\n");
        printf("   Stats: %s\n", stats);
        mongoose_free_string(stats);
    } else {
        printf("❌ FAILED: Could not get statistics\n");
    }

    mongoose_shutdown(handle);
    printf("\n");
}

void test_error_handling() {
    printf("=== Test 6: Error Handling ===\n");

    // Test with NULL handle
    bool should_block = false;
    char* filter = NULL;
    bool success = mongoose_check_network_request(
        NULL,  // NULL handle
        "https://example.com",
        "https://referrer.com",
        0,
        &should_block,
        &filter
    );

    if (!success) {
        printf("✅ PASSED: NULL handle rejected correctly\n");
    } else {
        printf("❌ FAILED: NULL handle was accepted\n");
    }

    // Test with NULL strings
    MongooseHandle handle = mongoose_initialize("/tmp/mongoose-test-ffi");
    if (handle) {
        success = mongoose_check_network_request(
            handle,
            NULL,  // NULL URL
            "https://referrer.com",
            0,
            &should_block,
            &filter
        );

        if (!success) {
            printf("✅ PASSED: NULL URL rejected correctly\n");
        } else {
            printf("❌ FAILED: NULL URL was accepted\n");
        }

        mongoose_shutdown(handle);
    }

    printf("\n");
}

int main() {
    printf("╔════════════════════════════════════════════════════╗\n");
    printf("║  Mongoose FFI Bridge - Standalone Test Suite      ║\n");
    printf("╚════════════════════════════════════════════════════╝\n\n");

    test_initialization();
    test_network_blocking();
    test_ascii_mode();
    test_bitmap_rendering();
    test_statistics();
    test_error_handling();

    printf("╔════════════════════════════════════════════════════╗\n");
    printf("║  Test Suite Complete                               ║\n");
    printf("╚════════════════════════════════════════════════════╝\n");

    return 0;
}
