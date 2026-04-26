//! macOS 平台实现

mod accessibility;
pub mod media;
mod window;

pub use accessibility::*;
pub use media::{get_media_metadata, get_playback_state, MediaMetadata, PlaybackState};
pub use window::get_frontmost_window_info_sync;
