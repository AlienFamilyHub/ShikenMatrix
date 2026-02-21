//! Version information FFI

use std::ffi::CString;
use std::os::raw::c_char;

/// Get the library version string
/// Returns a pointer to a null-terminated UTF-8 string
/// The caller must free the returned string using sm_string_free
#[no_mangle]
pub extern "C" fn sm_get_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    match CString::new(version) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
