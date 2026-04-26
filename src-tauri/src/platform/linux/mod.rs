//! Linux 平台占位实现。

use super::WindowInfo;

#[derive(Debug, Clone, PartialEq)]
pub struct MediaMetadata {
    pub bundle_identifier: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub artwork_data: Option<Vec<u8>>,
    pub artwork_mime_type: Option<String>,
    pub content_item_identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackState {
    pub playing: bool,
    pub playback_rate: f64,
    pub elapsed_time: f64,
}

pub fn request_permissions() -> Result<bool, String> {
    Ok(false)
}

pub fn check_permissions() -> bool {
    false
}

pub fn get_frontmost_window() -> Result<WindowInfo, String> {
    Err("Linux window monitoring is not implemented".to_string())
}

pub fn get_all_windows() -> Result<Vec<WindowInfo>, String> {
    Ok(Vec::new())
}

pub fn get_media_metadata() -> Result<Option<MediaMetadata>, String> {
    Ok(None)
}

pub fn get_playback_state() -> Result<Option<PlaybackState>, String> {
    Ok(None)
}
