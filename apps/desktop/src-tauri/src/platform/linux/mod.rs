//! Linux 窗口采集（X11 / 合成器 IPC）和 MPRIS 媒体采集。

mod media;
mod window;

pub use media::{MediaMetadata, PlaybackState, get_media_info};
pub use window::{check_permissions, get_all_windows, get_frontmost_window, request_permissions};
