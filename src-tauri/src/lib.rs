mod platform;

use platform::{
    WindowInfo,
    get_media_metadata, get_playback_state, MediaMetadata, PlaybackState
};

// Platform-specific window functions
#[cfg(target_os = "macos")]
use platform::macos::{
    get_frontmost_window_info_sync,
    request_accessibility_permission, check_accessibility_permission,
};

#[cfg(target_os = "windows")]
use platform::windows::{
    get_frontmost_window as get_frontmost_window_info_sync,
    request_permissions as request_accessibility_permission,
    check_permissions as check_accessibility_permission,
};

/// 异步获取前台窗口信息
#[tauri::command]
async fn get_frontmost_window() -> Result<WindowInfo, String> {
    tokio::task::spawn_blocking(|| {
        get_frontmost_window_info_sync()
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

/// 异步请求权限
#[tauri::command]
async fn request_permissions() -> Result<bool, String> {
    tokio::task::spawn_blocking(|| {
        request_accessibility_permission()
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

/// 检查权限（轻量操作，保持同步）
#[tauri::command]
fn check_permissions() -> bool {
    check_accessibility_permission()
}

/// 异步获取媒体元数据
#[tauri::command]
async fn get_media_metadata_cmd() -> Result<Option<MediaMetadata>, String> {
    tokio::task::spawn_blocking(|| {
        get_media_metadata()
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

/// 异步获取播放状态
#[tauri::command]
async fn get_playback_state_cmd() -> Result<Option<PlaybackState>, String> {
    tokio::task::spawn_blocking(|| {
        get_playback_state()
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_frontmost_window,
            request_permissions,
            check_permissions,
            get_media_metadata_cmd,
            get_playback_state_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
