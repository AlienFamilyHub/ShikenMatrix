//! FFI functions for reporter lifecycle management

use super::types::{SmConfig, SmReporter, SmStatus, SmLogCallback, SmWindowDataCallback, SmMediaDataCallback, SmLogLevel};
use crate::services::Reporter;
use std::ffi::CStr;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{info, error};
use tokio::runtime::Runtime;

/// Global storage for the reporter instance
static GLOBAL_REPORTER: Mutex<Option<ReporterHandle>> = Mutex::new(None);

/// Global tokio runtime (using OnceLock for safe initialization)
static GLOBAL_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Global logging initialized flag
static LOGGING_INITIALIZED: OnceLock<()> = OnceLock::new();

type ReporterHandle = Arc<Reporter>;

/// Initialize logging to stdout/stderr (captured by Xcode)
/// Reads log level from config file: ~/.shikenmatrix/config.toml
fn init_logging() {
    LOGGING_INITIALIZED.get_or_init(|| {
        // Try to get log level from config file, fall back to env var, then default to "info"
        let log_level = crate::services::get_log_level();
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level));

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();
    });
}

/// Initialize the global tokio runtime
fn get_runtime() -> &'static Runtime {
    GLOBAL_RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create tokio runtime")
    })
}

/// Start the reporter with the given configuration
///
/// # Arguments
/// * `config` - Pointer to SmConfig struct (will not be modified or freed)
///
/// # Returns
/// * Non-null pointer - Handle to the running reporter (opaque)
/// * Null pointer - Failed to start reporter (config was null or reporter already running)
///
/// # Safety
/// The returned pointer must be passed to sm_reporter_stop to clean up resources
#[no_mangle]
pub extern "C" fn sm_reporter_start(config: *const SmConfig) -> *mut SmReporter {
    // Initialize logging first so all output is visible in Xcode console
    init_logging();
    info!(">>> FFI: sm_reporter_start called");

    if config.is_null() {
        error!("sm_reporter_start: null config pointer");
        return std::ptr::null_mut();
    }

    // Check if reporter is already running
    {
        let guard = GLOBAL_REPORTER.lock().unwrap();
        if guard.is_some() {
            error!("sm_reporter_start: reporter already running");
            return std::ptr::null_mut();
        }
    }

    let (enabled, ws_url, token, enable_media_reporting) = unsafe {
        let cfg = &*config;
        (
            cfg.enabled,
            CStr::from_ptr(cfg.ws_url).to_string_lossy().to_string(),
            CStr::from_ptr(cfg.token).to_string_lossy().to_string(),
            cfg.enable_media_reporting,
        )
    };

    // Set environment variable for media reporting BEFORE spawning threads
    if enable_media_reporting {
        std::env::set_var("ENABLE_MEDIA_REPORTING", "1");
        info!(">>> Media reporting ENABLED");
    } else {
        std::env::set_var("ENABLE_MEDIA_REPORTING", "0");
        info!(">>> Media reporting DISABLED");
    }

    let reporter_config = crate::services::ReporterConfig {
        enabled,
        ws_url: ws_url.clone(),
        token: token.clone(),
        enable_media_reporting,
    };

    info!(">>> Creating reporter with config:");
    info!(">>>   enabled: {}", enabled);
    info!(">>>   ws_url: {}", ws_url);
    info!(">>>   token length: {}", token.len());
    info!(">>>   enable_media_reporting: {}", enable_media_reporting);

    // Create reporter using the runtime handle
    let rt = get_runtime();
    let handle = rt.handle().clone();
    let reporter = Reporter::new_with_handle(reporter_config, handle);

    // Store the reporter globally
    {
        let mut guard = GLOBAL_REPORTER.lock().unwrap();
        *guard = Some(Arc::new(reporter));
    }

    info!("Reporter started successfully");

    // Return a non-null pointer as a handle (the actual value doesn't matter,
    // we just use it as a token to indicate the reporter is running)
    Box::into_raw(Box::new(())) as *mut SmReporter
}

/// Stop the running reporter
///
/// # Arguments
/// * `handle` - Handle returned by sm_reporter_start
///
/// # Returns
/// * `true` - Reporter stopped successfully
/// * `false` - Failed to stop (invalid handle or reporter not running)
#[no_mangle]
pub extern "C" fn sm_reporter_stop(_handle: *mut SmReporter) -> bool {
    // We ignore the actual handle value and just check if a reporter is running
    let mut guard = GLOBAL_REPORTER.lock().unwrap();
    if guard.is_some() {
        *guard = None;
        info!("Reporter stopped successfully");
        true
    } else {
        error!("sm_reporter_stop: no reporter running");
        false
    }
}

/// Get the current status of the reporter
///
/// # Arguments
/// * `handle` - Handle returned by sm_reporter_start (ignored but kept for API consistency)
///
/// # Returns
/// * SmStatus struct containing the current status
#[no_mangle]
pub extern "C" fn sm_reporter_get_status(_handle: *const SmReporter) -> SmStatus {
    let guard = GLOBAL_REPORTER.lock().unwrap();

    let is_running = guard.is_some();
    // Get actual WebSocket connection status from the reporter
    let is_connected = guard.as_ref().map(|r| r.is_connected()).unwrap_or(false);

    SmStatus {
        is_running,
        is_connected,
        last_error: std::ptr::null_mut(),
    }
}

/// Check if the reporter is currently running
///
/// # Returns
/// * `true` - Reporter is running
/// * `false` - Reporter is not running
#[no_mangle]
pub extern "C" fn sm_reporter_is_running() -> bool {
    let guard = GLOBAL_REPORTER.lock().unwrap();
    guard.is_some()
}

/// Set log callback for receiving formatted logs from backend
///
/// # Arguments
/// * `callback` - Function pointer to log callback
/// * `user_data` - User data value to pass to callback
#[no_mangle]
pub extern "C" fn sm_reporter_set_log_callback(callback: SmLogCallback, user_data: usize) {
    let guard = GLOBAL_REPORTER.lock().unwrap();
    if let Some(reporter) = guard.as_ref() {
        // Wrap the callback to convert u8 to SmLogLevel
        let wrapped_callback: extern "C" fn(u8, *const std::os::raw::c_char, usize) = 
            unsafe { std::mem::transmute(callback) };
        reporter.set_log_callback(Some(wrapped_callback), user_data);
        info!("Log callback registered");
    } else {
        error!("sm_reporter_set_log_callback: no reporter running");
    }
}

/// Set window data callback for receiving window information
///
/// # Arguments
/// * `callback` - Function pointer to window data callback
/// * `user_data` - User data value to pass to callback
#[no_mangle]
pub extern "C" fn sm_reporter_set_window_callback(callback: SmWindowDataCallback, user_data: usize) {
    let guard = GLOBAL_REPORTER.lock().unwrap();
    if let Some(reporter) = guard.as_ref() {
        reporter.set_window_callback(Some(callback), user_data);
        info!("Window callback registered");
    } else {
        error!("sm_reporter_set_window_callback: no reporter running");
    }
}

/// Set media data callback for receiving media playback information
///
/// # Arguments
/// * `callback` - Function pointer to media data callback
/// * `user_data` - User data value to pass to callback
#[no_mangle]
pub extern "C" fn sm_reporter_set_media_callback(callback: SmMediaDataCallback, user_data: usize) {
    let guard = GLOBAL_REPORTER.lock().unwrap();
    if let Some(reporter) = guard.as_ref() {
        reporter.set_media_callback(Some(callback), user_data);
        info!("Media callback registered");
    } else {
        error!("sm_reporter_set_media_callback: no reporter running");
    }
}

// Note: We don't implement sm_reporter_free since the handle is just a token
// and the actual cleanup happens in sm_reporter_stop
