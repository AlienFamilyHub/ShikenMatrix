//! macOS 平台实现

mod accessibility;
pub mod media;
mod window;

pub use accessibility::*;
pub use media::{MediaMetadata, PlaybackState};
pub use window::get_frontmost_window_info_sync;
