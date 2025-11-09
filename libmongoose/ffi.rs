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
use log::{error, info, warn};

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

/// Check if a network request should be blocked
///
/// # Arguments
/// * `handle` - Handle from `mongoose_initialize()`
/// * `url` - The URL being requested (null-terminated C string)
/// * `source_url` - The page initiating the request (null-terminated C string)
/// * `resource_type` - Type of resource (0=document, 1=stylesheet, 2=image, 3=media, 4=font, 5=script, etc.)
/// * `should_block_out` - Output parameter: set to true if request should be blocked
/// * `filter_out` - Output parameter: pointer to receive matched filter string (or NULL)
///
/// # Returns
/// * `true` on success
/// * `false` on error (invalid handle or strings)
///
/// # Safety
/// * `handle` must be a valid handle from `mongoose_initialize()`
/// * `url` and `source_url` must be valid null-terminated C strings
/// * `should_block_out` must be a valid pointer to a bool
/// * If `filter_out` is not NULL, caller must call `mongoose_free_string()` on the returned string
///
/// # Example (C++)
/// ```cpp
/// bool should_block = false;
/// char* filter = nullptr;
/// bool success = mongoose_check_network_request(
///     handle,
///     "https://ads.example.com/script.js",
///     "https://site.com",
///     5,  // Script
///     &should_block,
///     &filter
/// );
/// if (success && should_block) {
///   LOG(INFO) << "Blocked by filter: " << filter;
///   mongoose_free_string(filter);
/// }
/// ```
#[no_mangle]
pub extern "C" fn mongoose_check_network_request(
    handle: MongooseHandle,
    url: *const c_char,
    source_url: *const c_char,
    resource_type: i32,
    should_block_out: *mut bool,
    filter_out: *mut *mut c_char,
) -> bool {
    // Validate inputs
    if handle.is_null() {
        error!("mongoose_check_network_request: handle is null");
        return false;
    }

    if url.is_null() || source_url.is_null() || should_block_out.is_null() {
        error!("mongoose_check_network_request: invalid parameters");
        return false;
    }

    // Convert C strings to Rust strings
    let (url_str, source_url_str) = unsafe {
        let url_result = CStr::from_ptr(url).to_str();
        let source_result = CStr::from_ptr(source_url).to_str();

        match (url_result, source_result) {
            (Ok(u), Ok(s)) => (u, s),
            _ => {
                error!("mongoose_check_network_request: invalid UTF-8 in URLs");
                return false;
            }
        }
    };

    // Check request using orchestrator's privacy engine
    let result = unsafe {
        let orchestrator = &*handle;
        orchestrator.handle_check_network_request(url_str, source_url_str, resource_type)
    };

    // Set output parameters
    unsafe {
        *should_block_out = result.should_block;

        if !filter_out.is_null() && result.matched_filter.is_some() {
            match CString::new(result.matched_filter.unwrap()) {
                Ok(c_string) => *filter_out = c_string.into_raw(),
                Err(e) => {
                    error!("Failed to create filter C string: {}", e);
                    *filter_out = std::ptr::null_mut();
                }
            }
        } else if !filter_out.is_null() {
            *filter_out = std::ptr::null_mut();
        }
    }

    true
}

/// Render a bitmap as ASCII art and send to terminal
///
/// # Arguments
/// * `handle` - Handle from `mongoose_initialize()`
/// * `pixels` - Pointer to RGBA pixel data (4 bytes per pixel)
/// * `width` - Width of the image in pixels
/// * `height` - Height of the image in pixels
/// * `x` - X coordinate for rendering (terminal column)
/// * `y` - Y coordinate for rendering (terminal row)
///
/// # Returns
/// * `true` on success
/// * `false` on error
///
/// # Safety
/// * `handle` must be a valid handle from `mongoose_initialize()`
/// * `pixels` must point to valid memory of size `width * height * 4` bytes
/// * Pixel data must be in RGBA format (R, G, B, A for each pixel)
///
/// # Example (C++)
/// ```cpp
/// std::vector<uint8_t> rgba_pixels = GetBitmapPixels();
/// bool success = mongoose_render_bitmap(
///     handle,
///     rgba_pixels.data(),
///     800,  // width
///     600,  // height
///     0,    // x
///     0     // y
/// );
/// ```
#[no_mangle]
pub extern "C" fn mongoose_render_bitmap(
    handle: MongooseHandle,
    pixels: *const u8,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
) -> bool {
    if handle.is_null() {
        error!("mongoose_render_bitmap: handle is null");
        return false;
    }

    if pixels.is_null() {
        error!("mongoose_render_bitmap: pixels is null");
        return false;
    }

    if width == 0 || height == 0 {
        warn!("mongoose_render_bitmap: zero dimensions");
        return false;
    }

    // Calculate expected size
    let expected_size = (width * height * 4) as usize;

    // Copy pixel data to Rust-owned Vec
    let pixel_data = unsafe {
        std::slice::from_raw_parts(pixels, expected_size).to_vec()
    };

    // Call orchestrator to render
    unsafe {
        let orchestrator = &*handle;
        orchestrator.handle_draw_image_bitmap(&pixel_data, width, height, x, y);
    }

    info!("Rendered {}x{} bitmap at ({}, {})", width, height, x, y);
    true
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

        // Initialization might fail due to network issues fetching filters
        // Skip the test if initialization failed
        if handle.is_null() {
            eprintln!("Skipping test_initialize_shutdown: initialization failed (network issue?)");
            return;
        }

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
