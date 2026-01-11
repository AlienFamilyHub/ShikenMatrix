//! ShikenMatrix - Window and Media Reporter Library
//!
//! This library provides functionality to monitor and report:
//! - Active window information
//! - Media playback state
//!
//! ## FFI API
//!
//! For native UI integration, use the `ffi` module which provides a C-compatible API:
//! - `ffi::config` - Configuration management
//! - `ffi::reporter` - Reporter lifecycle management
//! - `ffi::types` - FFI-compatible types
//!
//! ## Usage as a Library
//!
//! ```rust
//! use shikenmatrix::services::{Reporter, ReporterConfig, load_config};
//!
//! let config = load_config();
//! if config.reporter.enabled {
//!     let reporter = Reporter::new(config.reporter);
//!     // Use reporter...
//! }
//! ```

pub mod ffi;
pub mod platform;
pub mod services;

// Re-export common types for convenience
pub use services::{Reporter, ReporterConfig, load_config, save_reporter_config};
