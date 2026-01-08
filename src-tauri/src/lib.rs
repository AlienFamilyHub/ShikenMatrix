mod platform;
mod services;

use platform::{
    WindowInfo,
    get_media_metadata, get_playback_state, MediaMetadata, PlaybackState
};
use services::{Reporter, ReporterConfig, load_config, save_reporter_config};
use std::sync::{Arc, RwLock};
use std::collections::HashSet;
use tauri::Manager;

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
async fn get_frontmost_window(
    reporter: tauri::State<'_, Arc<RwLock<Option<Reporter>>>>,
) -> Result<WindowInfo, String> {
    let window_info = tokio::task::spawn_blocking(|| {
        get_frontmost_window_info_sync()
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))??;

    // 上报窗口信息（仅在变化时发送）
    if let Ok(guard) = reporter.read() {
        if let Some(r) = guard.as_ref() {
            r.send_window_info(&window_info);
        }
    }

    Ok(window_info)
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
async fn get_media_metadata_cmd(
    _reporter: tauri::State<'_, Arc<RwLock<Option<Reporter>>>>,
) -> Result<Option<MediaMetadata>, String> {
    tokio::task::spawn_blocking(|| {
        get_media_metadata()
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

/// 异步获取播放状态
#[tauri::command]
async fn get_playback_state_cmd(
    reporter: tauri::State<'_, Arc<RwLock<Option<Reporter>>>>,
    uploaded_artworks: tauri::State<'_, Arc<RwLock<HashSet<String>>>>,
) -> Result<Option<PlaybackState>, String> {
    let state = tokio::task::spawn_blocking(|| {
        get_playback_state()
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))??;

    // 如果有播放状态和媒体信息，上报（仅在变化时发送）
    if let Some(ref playback_state) = state {
        if playback_state.playing {
            if let Ok(Some(metadata)) = get_media_metadata() {
                // 检查是否有封面需要上传
                if let Some(content_id) = &metadata.content_item_identifier {
                    if let Some(artwork_data) = &metadata.artwork_data {
                        if let Some(mime_type) = &metadata.artwork_mime_type {
                            // 检查是否已经上传过这个封面
                            let should_upload = {
                                let uploaded = uploaded_artworks.read().map_err(|e| format!("获取锁失败: {}", e))?;
                                !uploaded.contains(content_id)
                            };

                            if should_upload {
                                // 上传封面
                                if let Ok(guard) = reporter.read() {
                                    if let Some(r) = guard.as_ref() {
                                        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
                                        match BASE64.decode(artwork_data) {
                                            Ok(artwork_bytes) => {
                                                println!("🖼️ 上传封面: {} ({} bytes)", content_id, artwork_bytes.len());
                                                r.upload_artwork(content_id.clone(), artwork_bytes, mime_type.clone());
                                                
                                                // 标记为已上传
                                                if let Ok(mut uploaded) = uploaded_artworks.write() {
                                                    uploaded.insert(content_id.clone());
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("❌ Base64 解码失败: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 上报媒体播放状态
                if let Ok(guard) = reporter.read() {
                    if let Some(r) = guard.as_ref() {
                        r.send_media_playback(&metadata, playback_state);
                    }
                }
            }
        }
    }

    Ok(state)
}

/// 更新上报配置（同时保存到 config.toml）
#[tauri::command]
async fn update_reporter_config(
    config: ReporterConfig,
    reporter: tauri::State<'_, Arc<RwLock<Option<Reporter>>>>,
) -> Result<(), String> {
    // 保存配置到文件
    save_reporter_config(&config)?;
    
    let mut reporter_lock = reporter.write().map_err(|e| format!("获取锁失败: {}", e))?;
    
    if config.enabled {
        // 如果启用，创建或更新 reporter
        if let Some(existing_reporter) = reporter_lock.as_ref() {
            existing_reporter.update_config(config);
        } else {
            *reporter_lock = Some(Reporter::new(config));
        }
        println!("✅ 上报功能已启用");
    } else {
        // 如果禁用，移除 reporter
        *reporter_lock = None;
        println!("❌ 上报功能已禁用");
    }
    
    Ok(())
}

/// 上传媒体封面
#[tauri::command]
async fn upload_media_artwork(
    content_item_identifier: String,
    artwork_base64: String,
    mime_type: String,
    reporter: tauri::State<'_, Arc<RwLock<Option<Reporter>>>>,
) -> Result<(), String> {
    // 解码 Base64
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    let artwork_data = BASE64.decode(&artwork_base64)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    // 上传封面
    if let Ok(guard) = reporter.read() {
        if let Some(r) = guard.as_ref() {
            r.upload_artwork(content_item_identifier, artwork_data, mime_type);
            return Ok(());
        }
    }

    Err("上报功能未启用".to_string())
}

/// 获取当前上报配置状态
#[tauri::command]
fn get_reporter_status(
    reporter: tauri::State<'_, Arc<RwLock<Option<Reporter>>>>,
) -> Result<bool, String> {
    Ok(reporter.read().map_err(|e| format!("获取锁失败: {}", e))?.is_some())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 启动时加载配置（但不立即创建 Reporter）
    let app_config = load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .manage(Arc::new(RwLock::new(None::<Reporter>)))
        .manage(Arc::new(RwLock::new(HashSet::<String>::new()))) // 已上传封面的缓存
        .setup(move |_app| {
            // 在 Tauri runtime 启动后创建 Reporter
            if app_config.reporter.enabled {
                println!("🚀 从配置文件加载上报设置");
                let reporter = Reporter::new(app_config.reporter.clone());
                let state = _app.state::<Arc<RwLock<Option<Reporter>>>>();
                if let Ok(mut guard) = state.write() {
                    *guard = Some(reporter);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_frontmost_window,
            request_permissions,
            check_permissions,
            get_media_metadata_cmd,
            get_playback_state_cmd,
            update_reporter_config,
            get_reporter_status,
            upload_media_artwork
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
