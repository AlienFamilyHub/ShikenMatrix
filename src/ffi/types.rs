//! FFI-compatible types for cross-platform interop

use std::ffi::c_char;

/// Configuration for the reporter
#[repr(C)]
pub struct SmConfig {
    /// Whether the reporter is enabled
    pub enabled: bool,
    /// WebSocket URL (null-terminated string, owned by Rust)
    pub ws_url: *mut c_char,
    /// Authentication token (null-terminated string, owned by Rust)
    pub token: *mut c_char,
}

/// Status of the reporter
#[repr(C)]
pub struct SmStatus {
    /// Whether the reporter is running
    pub is_running: bool,
    /// Whether the WebSocket is connected
    pub is_connected: bool,
    /// Last error message (null-terminated string, owned by Rust, null if no error)
    pub last_error: *mut c_char,
}

/// Window information for FFI
#[repr(C)]
pub struct SmWindowInfo {
    /// Window title (null-terminated string, owned by Rust)
    pub title: *mut c_char,
    /// Process name (null-terminated string, owned by Rust)
    pub process_name: *mut c_char,
    /// App ID/bundle ID (null-terminated string, owned by Rust, null if not available)
    pub app_id: *mut c_char,
    /// Process ID
    pub pid: i32,
    /// Whether icon data is available
    pub has_icon: bool,
    /// Icon data (PNG format, owned by Rust, null if has_icon is false)
    pub icon_data: *const u8,
    /// Size of icon data in bytes
    pub icon_size: usize,
}

/// Opaque handle for Reporter instance
#[repr(C)]
pub struct SmReporter;

// Safety: SmReporter is an opaque type for FFI only
// It's never directly constructed or deconstructed by foreign code
