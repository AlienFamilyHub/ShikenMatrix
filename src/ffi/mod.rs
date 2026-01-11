//! FFI layer for cross-platform native UI integration
//!
//! This module provides a C-compatible API for native UI frameworks:
//! - macOS: SwiftUI (via Swift)
//! - Windows: WinUI 3 (via C#)
//!
//! All functions are `extern "C"` with no_mangle to ensure stable C ABI.

pub mod accessibility;
pub mod config;
pub mod reporter;
pub mod types;

// Re-export the main FFI API
pub use types::{SmConfig, SmStatus, SmWindowInfo, SmReporter};
