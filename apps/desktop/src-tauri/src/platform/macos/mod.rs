//! macOS 平台实现

mod accessibility;
pub mod media;
mod window;

pub use accessibility::*;
pub use media::{MediaMetadata, PlaybackState, get_media_metadata, get_playback_state};
pub use window::get_frontmost_window_info_sync;
