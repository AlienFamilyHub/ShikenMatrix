use super::Monitor;
use super::types::LogLevel;
use crate::platform::{MediaMetadata, PlaybackState, WindowInfo};
use std::sync::atomic::Ordering;

#[cfg(target_os = "windows")]
use base64::Engine;
#[cfg(target_os = "windows")]
use base64::engine::general_purpose::STANDARD as BASE64;

impl Monitor {
    pub(super) fn start_monitoring(&self) {
        let monitor = self.clone();

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            monitor.emit_log(LogLevel::Info, "窗口监控已启动");

            let mut permission_warned = false;
            let mut last_window_info: Option<WindowInfo> = None;
            let mut last_media_metadata: Option<MediaMetadata> = None;
            let mut last_playback_state: Option<PlaybackState> = None;

            while monitor.is_running.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(1));

                monitor.poll_platform(
                    &mut last_window_info,
                    &mut permission_warned,
                    &mut last_media_metadata,
                    &mut last_playback_state,
                );
            }
        });
    }

    fn poll_platform(
        &self,
        last_window_info: &mut Option<WindowInfo>,
        permission_warned: &mut bool,
        last_media_metadata: &mut Option<MediaMetadata>,
        last_playback_state: &mut Option<PlaybackState>,
    ) {
        #[cfg(target_os = "macos")]
        objc2::rc::autoreleasepool(|_| {
            self.poll_window(last_window_info, permission_warned);
            self.poll_media(last_media_metadata, last_playback_state);
        });

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            self.poll_window(last_window_info, permission_warned);
            self.poll_media(last_media_metadata, last_playback_state);
        }
    }

    fn poll_window(&self, last_window_info: &mut Option<WindowInfo>, permission_warned: &mut bool) {
        match current_window_info() {
            Ok(window_info) if last_window_info.as_ref() != Some(&window_info) => {
                self.emit_log(
                    LogLevel::Info,
                    format!(
                        "获取到窗口信息: {} ({})",
                        window_info.title, window_info.process_name
                    ),
                );
                self.emit_window(&window_info);
                self.send_window_info(&window_info);
                *permission_warned = false;
                *last_window_info = Some(window_info);
            }
            Ok(_) => {}
            Err(error) if !*permission_warned => {
                self.emit_log(LogLevel::Warn, format!("获取窗口信息失败: {error}"));
                *permission_warned = true;
            }
            Err(_) => {}
        }
    }

    fn poll_media(
        &self,
        last_media_metadata: &mut Option<MediaMetadata>,
        last_playback_state: &mut Option<PlaybackState>,
    ) {
        let Ok(Some(metadata)) = current_media_metadata() else {
            return;
        };
        let Ok(Some(state)) = current_playback_state() else {
            return;
        };

        let metadata_changed = last_media_metadata.as_ref() != Some(&metadata);
        let state_changed = last_playback_state.as_ref() != Some(&state);

        if !metadata_changed && !state_changed {
            return;
        }

        self.emit_media(&metadata, &state, extract_artwork_data(&metadata));
        self.maybe_upload_media(&metadata, &state, metadata_changed);

        *last_media_metadata = Some(metadata);
        *last_playback_state = Some(state);
    }

    fn maybe_upload_media(&self, metadata: &MediaMetadata, state: &PlaybackState, changed: bool) {
        #[cfg(target_os = "macos")]
        {
            self.send_media_playback(metadata, state);

            if !changed {
                return;
            }

            if let (Some(artwork), Some(mime_type), Some(content_id)) = (
                metadata.artwork_data.as_ref(),
                metadata.artwork_mime_type.as_ref(),
                metadata.content_item_identifier.as_ref(),
            ) {
                let needs_upload = self
                    .artwork_urls
                    .read()
                    .map(|urls| !urls.contains_key(content_id))
                    .unwrap_or(true);

                if needs_upload {
                    self.upload_artwork(content_id.clone(), artwork.to_vec(), mime_type.clone());
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (metadata, state, changed);
        }
    }
}

#[cfg(target_os = "macos")]
fn current_window_info() -> Result<WindowInfo, String> {
    crate::platform::macos::get_frontmost_window_info_sync()
}

#[cfg(target_os = "windows")]
fn current_window_info() -> Result<WindowInfo, String> {
    crate::platform::windows::get_frontmost_window()
}

#[cfg(target_os = "linux")]
fn current_window_info() -> Result<WindowInfo, String> {
    crate::platform::linux::get_frontmost_window()
}

#[cfg(target_os = "macos")]
fn current_media_metadata() -> Result<Option<MediaMetadata>, String> {
    crate::platform::macos::get_media_metadata()
}

#[cfg(target_os = "windows")]
fn current_media_metadata() -> Result<Option<MediaMetadata>, String> {
    crate::platform::windows::get_media_metadata()
}

#[cfg(target_os = "linux")]
fn current_media_metadata() -> Result<Option<MediaMetadata>, String> {
    crate::platform::linux::get_media_metadata()
}

#[cfg(target_os = "macos")]
fn current_playback_state() -> Result<Option<PlaybackState>, String> {
    crate::platform::macos::get_playback_state()
}

#[cfg(target_os = "windows")]
fn current_playback_state() -> Result<Option<PlaybackState>, String> {
    crate::platform::windows::get_playback_state()
}

#[cfg(target_os = "linux")]
fn current_playback_state() -> Result<Option<PlaybackState>, String> {
    crate::platform::linux::get_playback_state()
}

#[cfg(target_os = "macos")]
fn extract_artwork_data(metadata: &MediaMetadata) -> Option<Vec<u8>> {
    metadata.artwork_data.as_ref().map(|data| data.to_vec())
}

#[cfg(target_os = "windows")]
fn extract_artwork_data(metadata: &MediaMetadata) -> Option<Vec<u8>> {
    metadata
        .artwork_data
        .as_ref()
        .and_then(|artwork| BASE64.decode(artwork).ok())
}

#[cfg(target_os = "linux")]
fn extract_artwork_data(_metadata: &MediaMetadata) -> Option<Vec<u8>> {
    None
}
