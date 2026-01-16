//! Windows 平台实现

pub mod media;
pub mod window;

pub use media::{get_media_metadata, get_playback_state, MediaMetadata, PlaybackState};
pub use window::{get_frontmost_window, get_all_windows};

/// 请求必要的权限 (Windows 通常不需要像 macOS 那样显式请求权限)
pub fn request_permissions() -> Result<bool, String> {
    Ok(true)
}

/// 检查权限状态
pub fn check_permissions() -> bool {
    true
}
