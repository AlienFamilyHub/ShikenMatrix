//! FFI functions for configuration management

use super::types::SmConfig;
use crate::services::{load_config, save_reporter_config, ReporterConfig};
use std::ffi::{CStr, CString, c_char};
use tracing::info;

/// Load configuration from file
///
/// Returns a pointer to SmConfig that must be freed with sm_config_free
#[no_mangle]
pub extern "C" fn sm_config_load() -> *mut SmConfig {
    let config = load_config();
    let reporter_cfg = config.reporter;

    let ws_url = CString::new(reporter_cfg.ws_url).unwrap_or_else(|_| CString::default()).into_raw();
    let token = CString::new(reporter_cfg.token).unwrap_or_else(|_| CString::default()).into_raw();

    Box::into_raw(Box::new(SmConfig {
        enabled: reporter_cfg.enabled,
        ws_url,
        token,
    }))
}

/// Save configuration to file
///
/// # Arguments
/// * `config` - Pointer to SmConfig struct (will not be modified or freed)
///
/// # Returns
/// * `true` - Configuration saved successfully
/// * `false` - Failed to save (config was null or save failed)
#[no_mangle]
pub extern "C" fn sm_config_save(config: *const SmConfig) -> bool {
    if config.is_null() {
        tracing::error!("sm_config_save: null config pointer");
        return false;
    }

    let (enabled, ws_url, token) = unsafe {
        let cfg = &*config;
        (
            cfg.enabled,
            CStr::from_ptr(cfg.ws_url).to_string_lossy().to_string(),
            CStr::from_ptr(cfg.token).to_string_lossy().to_string(),
        )
    };

    let reporter_config = ReporterConfig {
        enabled,
        ws_url,
        token,
    };

    match save_reporter_config(&reporter_config) {
        Ok(_) => {
            info!("Configuration saved successfully");
            true
        }
        Err(e) => {
            tracing::error!("Failed to save configuration: {}", e);
            false
        }
    }
}

/// Free a SmConfig struct created by sm_config_load
///
/// # Arguments
/// * `config` - Pointer to SmConfig to free (must not be null)
#[no_mangle]
pub extern "C" fn sm_config_free(config: *mut SmConfig) {
    if config.is_null() {
        return;
    }

    unsafe {
        let cfg = &mut *config;
        // Free the owned strings
        if !cfg.ws_url.is_null() {
            let _ = CString::from_raw(cfg.ws_url);
        }
        if !cfg.token.is_null() {
            let _ = CString::from_raw(cfg.token);
        }
        // Free the struct itself
        drop(Box::from_raw(config));
    }
}

/// Free a string allocated by Rust
///
/// This should be used for any *mut c_char returned from other FFI functions
/// when the caller is finished with it.
///
/// # Arguments
/// * `s` - Pointer to string to free (safe if null)
#[no_mangle]
pub extern "C" fn sm_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        };
    }
}
