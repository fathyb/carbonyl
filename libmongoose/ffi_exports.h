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
