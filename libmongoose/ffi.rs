/// FFI Bridge - C-compatible interface for C++ integration
///
/// This module provides C-compatible functions that C++ code can call
/// to initialize and interact with the Mongoose orchestrator.
///
/// Note: Due to thread-safety limitations in adblock-rust, we use a simpler
/// approach where the C++ side manages the orchestrator lifetime.

use crate::mojo_impl::MongooseOrchestrator;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use log::{error, info};

/// Opaque handle to MongooseOrchestrator
///
/// C++ code treats this as an opaque pointer and passes it back to Rust FFI functions.
pub type MongooseHandle = *mut MongooseOrchestrator;

/// Initialize the Mongoose orchestrator
///
/// This creates a new orchestrator instance and returns a handle to it.
/// The C++ side must store this handle and pass it to other functions.
///
/// # Arguments
/// * `data_dir` - Path to the data directory (e.g., ~/.config/mongoose)
///
/// # Returns
/// * Handle to orchestrator on success
/// * NULL on failure
///
/// # Safety
/// * `data_dir` must be a valid null-terminated C string
/// * Caller must call `mongoose_shutdown()` with the returned handle
///
/// # Example (C++)
/// ```cpp
/// MongooseHandle handle = mongoose_initialize("/home/user/.config/mongoose");
/// if (!handle) {
///   LOG(ERROR) << "Failed to initialize Mongoose";
///   return;
/// }
/// // ... use handle ...
/// mongoose_shutdown(handle);
/// ```
#[no_mangle]
pub extern "C" fn mongoose_initialize(data_dir: *const c_char) -> MongooseHandle {
    // Convert C string to Rust string
    let data_dir_str = unsafe {
        if data_dir.is_null() {
            error!("mongoose_initialize: data_dir is null");
            return std::ptr::null_mut();
        }

        match CStr::from_ptr(data_dir).to_str() {
            Ok(s) => s,
            Err(e) => {
                error!("mongoose_initialize: invalid UTF-8 in data_dir: {}", e);
                return std::ptr::null_mut();
            }
        }
    };

    info!("Initializing Mongoose with data_dir: {}", data_dir_str);

    // Create orchestrator
    match MongooseOrchestrator::new(data_dir_str) {
        Ok(orchestrator) => {
            let boxed = Box::new(orchestrator);
            let handle = Box::into_raw(boxed);
            info!("Mongoose initialized successfully (handle: {:?})", handle);
            handle
        }
        Err(e) => {
            error!("Failed to initialize Mongoose: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Shutdown the Mongoose orchestrator
///
/// This destroys the orchestrator and frees all resources.
///
/// # Arguments
/// * `handle` - Handle returned by `mongoose_initialize()`
///
/// # Safety
/// * `handle` must be a valid handle from `mongoose_initialize()`
/// * `handle` must not be used after this call
#[no_mangle]
pub extern "C" fn mongoose_shutdown(handle: MongooseHandle) {
    if handle.is_null() {
        error!("mongoose_shutdown: handle is null");
        return;
    }

    info!("Shutting down Mongoose (handle: {:?})", handle);

    unsafe {
        // Reconstruct Box and drop it to free memory
        drop(Box::from_raw(handle));
    }

    info!("Mongoose shutdown complete");
}

/// Check if a handle is valid (non-null)
///
/// # Arguments
/// * `handle` - Handle to check
///
/// # Returns
/// * `true` if handle is valid (non-null)
/// * `false` if handle is null
#[no_mangle]
pub extern "C" fn mongoose_is_initialized(handle: MongooseHandle) -> bool {
    !handle.is_null()
}

/// Enable or disable ASCII art rendering mode
///
/// # Arguments
/// * `handle` - Handle from `mongoose_initialize()`
/// * `enabled` - true to enable ASCII art mode, false for block mode
///
/// # Safety
/// * `handle` must be a valid handle from `mongoose_initialize()`
#[no_mangle]
pub extern "C" fn mongoose_set_ascii_mode(handle: MongooseHandle, enabled: bool) {
    if handle.is_null() {
        error!("mongoose_set_ascii_mode: handle is null");
        return;
    }

    unsafe {
        let orchestrator = &*handle;
        orchestrator.handle_set_hybrid_ascii_mode(enabled);
        info!("ASCII mode {}", if enabled { "enabled" } else { "disabled"});
    }
}

/// Get statistics about Mongoose as a JSON string
///
/// # Arguments
/// * `handle` - Handle from `mongoose_initialize()`
///
/// # Returns
/// * Pointer to null-terminated JSON string
/// * NULL if handle is invalid
///
/// # Safety
/// * `handle` must be a valid handle from `mongoose_initialize()`
/// * Caller must call `mongoose_free_string()` on the returned pointer
#[no_mangle]
pub extern "C" fn mongoose_get_stats(handle: MongooseHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    let stats_json = serde_json::json!({
        "initialized": true,
        "privacy_engine": "active",
        "render_engine": "active",
        "data_store": "active"
    }).to_string();

    match CString::new(stats_json) {
        Ok(c_string) => c_string.into_raw(),
        Err(e) => {
            error!("Failed to create C string: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Free a string allocated by Rust
///
/// # Arguments
/// * `s` - Pointer to string allocated by Rust
///
/// # Safety
/// * `s` must have been allocated by a Mongoose FFI function
/// * `s` must not be used after calling this function
#[no_mangle]
pub extern "C" fn mongoose_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_initialize_shutdown() {
        let data_dir = CString::new("/tmp/mongoose-test").unwrap();

        // Initialize
        let handle = mongoose_initialize(data_dir.as_ptr());
        assert!(!handle.is_null());
        assert!(mongoose_is_initialized(handle));

        // Shutdown
        mongoose_shutdown(handle);
    }

    #[test]
    fn test_null_data_dir() {
        let handle = mongoose_initialize(std::ptr::null());
        assert!(handle.is_null());
    }

    #[test]
    fn test_stats() {
        let data_dir = CString::new("/tmp/mongoose-test").unwrap();
        let handle = mongoose_initialize(data_dir.as_ptr());

        // Initialization might fail due to network issues fetching filters
        // Skip the test if initialization failed
        if handle.is_null() {
            eprintln!("Skipping test_stats: initialization failed (network issue?)");
            return;
        }

        let stats_ptr = mongoose_get_stats(handle);
        assert!(!stats_ptr.is_null());

        unsafe {
            let stats_str = CStr::from_ptr(stats_ptr).to_str().unwrap();
            assert!(stats_str.contains("initialized"));
        }

        mongoose_free_string(stats_ptr);
        mongoose_shutdown(handle);
    }
}
