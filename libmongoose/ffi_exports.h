/*
 * Copyright 2025 Mongoose Browser Project
 * FFI Exports - C header for C++ integration
 *
 * This header provides the C-compatible interface for C++ code to call
 * into the Rust libmongoose library.
 */

#ifndef LIBMONGOOSE_FFI_EXPORTS_H_
#define LIBMONGOOSE_FFI_EXPORTS_H_

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// Types
// ============================================================================

/**
 * Opaque handle to the Mongoose orchestrator.
 *
 * This handle is returned by mongoose_initialize() and must be passed to
 * all other Mongoose functions. Treat it as an opaque pointer.
 */
typedef void* MongooseHandle;

// ============================================================================
// Initialization and Lifecycle
// ============================================================================

/**
 * Initialize the Mongoose orchestrator.
 *
 * Creates a new Mongoose orchestrator instance with all services initialized
 * (privacy engine, rendering engine, data store).
 *
 * @param data_dir Path to the data directory (e.g., ~/.config/mongoose)
 * @return Handle to orchestrator on success, NULL on failure
 *
 * Example:
 *   MongooseHandle mongoose = mongoose_initialize("/home/user/.config/mongoose");
 *   if (!mongoose) {
 *     LOG(ERROR) << "Failed to initialize Mongoose";
 *     return;
 *   }
 *   // ... use mongoose ...
 *   mongoose_shutdown(mongoose);
 */
MongooseHandle mongoose_initialize(const char* data_dir);

/**
 * Shutdown the Mongoose orchestrator.
 *
 * Destroys the orchestrator and frees all resources. The handle must not
 * be used after calling this function.
 *
 * @param handle Handle returned by mongoose_initialize()
 */
void mongoose_shutdown(MongooseHandle handle);

/**
 * Check if a Mongoose handle is valid.
 *
 * @param handle Handle to check
 * @return true if handle is valid (non-NULL), false otherwise
 */
bool mongoose_is_initialized(MongooseHandle handle);

// ============================================================================
// Privacy Engine
// ============================================================================

/**
 * Check if a network request should be blocked.
 *
 * This function calls the Mongoose privacy engine to check if a request
 * matches any blocking filters. It must return quickly (microseconds) as
 * it's called synchronously from the network stack.
 *
 * @param handle Handle from mongoose_initialize()
 * @param url The URL being requested (null-terminated C string)
 * @param source_url The page initiating the request (null-terminated C string)
 * @param resource_type Type of resource:
 *        0=document, 1=stylesheet, 2=image, 3=media, 4=font, 5=script,
 *        6=XHR, 7=WebSocket, 8=ping, etc.
 * @param should_block_out Output: set to true if request should be blocked
 * @param filter_out Output: pointer to receive matched filter string (or NULL).
 *        If non-NULL and a filter matches, this will be set to an allocated
 *        string that must be freed with mongoose_free_string().
 * @return true on success, false on error (invalid handle or strings)
 *
 * Example:
 *   bool should_block = false;
 *   char* filter = nullptr;
 *   bool success = mongoose_check_network_request(
 *       handle,
 *       "https://ads.example.com/script.js",
 *       "https://site.com",
 *       5,  // Script
 *       &should_block,
 *       &filter
 *   );
 *   if (success && should_block) {
 *     LOG(INFO) << "Blocked by filter: " << filter;
 *     mongoose_free_string(filter);
 *   }
 */
bool mongoose_check_network_request(
    MongooseHandle handle,
    const char* url,
    const char* source_url,
    int32_t resource_type,
    bool* should_block_out,
    char** filter_out
);

// ============================================================================
// Rendering Engine
// ============================================================================

/**
 * Enable or disable hybrid ASCII art rendering mode.
 *
 * When enabled, images are captured and sent to the Rust rendering engine
 * for high-fidelity ASCII art conversion using chafa.
 *
 * When disabled (default), images are rendered using fast Unicode blocks
 * for 60fps performance.
 *
 * @param handle Handle from mongoose_initialize()
 * @param enabled true to enable ASCII mode, false for block mode
 *
 * Example:
 *   mongoose_set_ascii_mode(mongoose, true);  // Enable chafa rendering
 */
void mongoose_set_ascii_mode(MongooseHandle handle, bool enabled);

/**
 * Render a bitmap as ASCII art and send to terminal.
 *
 * This function captures a rendered bitmap and sends it to the Rust
 * rendering engine for ASCII art conversion using chafa. The bitmap
 * is stamped onto the frame buffer at the specified position.
 *
 * @param handle Handle from mongoose_initialize()
 * @param pixels Pointer to RGBA pixel data (4 bytes per pixel: R, G, B, A)
 * @param width Width of the image in pixels
 * @param height Height of the image in pixels
 * @param x X coordinate for rendering (terminal column)
 * @param y Y coordinate for rendering (terminal row)
 * @return true on success, false on error
 *
 * Example:
 *   std::vector<uint8_t> rgba_pixels = GetBitmapPixels();
 *   bool success = mongoose_render_bitmap(
 *       handle,
 *       rgba_pixels.data(),
 *       800,  // width
 *       600,  // height
 *       0,    // x
 *       0     // y
 *   );
 */
bool mongoose_render_bitmap(
    MongooseHandle handle,
    const uint8_t* pixels,
    uint32_t width,
    uint32_t height,
    int32_t x,
    int32_t y
);

// ============================================================================
// Statistics and Debugging
// ============================================================================

/**
 * Get statistics about Mongoose as a JSON string.
 *
 * The returned string must be freed with mongoose_free_string().
 *
 * @param handle Handle from mongoose_initialize()
 * @return Pointer to null-terminated JSON string, or NULL if handle is invalid
 *
 * Example:
 *   char* stats = mongoose_get_stats(mongoose);
 *   if (stats) {
 *     LOG(INFO) << "Mongoose stats: " << stats;
 *     mongoose_free_string(stats);
 *   }
 */
char* mongoose_get_stats(MongooseHandle handle);

/**
 * Free a string allocated by Rust.
 *
 * This must be called on any string returned by Mongoose FFI functions
 * (e.g., mongoose_get_stats).
 *
 * @param s Pointer to string to free
 *
 * Example:
 *   char* stats = mongoose_get_stats(mongoose);
 *   // ... use stats ...
 *   mongoose_free_string(stats);
 */
void mongoose_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif  // LIBMONGOOSE_FFI_EXPORTS_H_
