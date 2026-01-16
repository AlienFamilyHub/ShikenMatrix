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
    /// Whether to enable media reporting
    pub enable_media_reporting: bool,
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

/// Log level for callback
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SmLogLevel {
    Info = 0,
    Warning = 1,
    Error = 2,
}

/// Callback function type for logs
pub type SmLogCallback = extern "C" fn(level: SmLogLevel, message: *const c_char, user_data: usize);

/// Callback function type for window data (with icon)
pub type SmWindowDataCallback = extern "C" fn(
    title: *const c_char,
    process_name: *const c_char,
    pid: u32,
    icon_data: *const u8,
    icon_size: usize,
    user_data: usize
);

/// Callback function type for media data (with artwork)
pub type SmMediaDataCallback = extern "C" fn(
    title: *const c_char,
    artist: *const c_char,
    album: *const c_char,
    duration: f64,
    elapsed_time: f64,
    playing: bool,
    artwork_data: *const u8,
    artwork_size: usize,
    user_data: usize
);
