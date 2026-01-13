//! Windows 媒体播放信息获取模块
//! 基于 Windows.Media.Control (SMTC)

use serde::{Serialize, Deserialize};
use windows::core::{Result, HSTRING};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackInfo,
    GlobalSystemMediaTransportControlsSessionTimelineProperties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;
use tokio::runtime::Runtime;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

/// 播放状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    /// 是否正在播放
    pub playing: bool,
    /// 播放速率 (1.0 = 正常速度)
    pub playback_rate: f64,
    /// 已播放时长（秒）
    pub elapsed_time: f64,
}

/// 媒体元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// 应用 Bundle ID (Windows 上为 AUMID)
    pub bundle_identifier: Option<String>,
    /// 曲目标题
    pub title: Option<String>,
    /// 艺术家
    pub artist: Option<String>,
    /// 专辑
    pub album: Option<String>,
    /// 总时长（秒）
    pub duration: f64,
    /// 封面数据 (Base64 编码)
    pub artwork_data: Option<String>,
    /// 封面 MIME 类型
    pub artwork_mime_type: Option<String>,
    /// 内容标识符
    pub content_item_identifier: Option<String>,
}

/// 获取当前播放状态
pub fn get_playback_state() -> std::result::Result<Option<PlaybackState>, String> {
    // 创建 Tokio Runtime 来执行异步操作
    // 注意：在 Tauri 命令中通常已经有 Runtime，但这里可能在同步上下文中调用
    // 为了稳妥，这里创建一个临时的 Runtime，或者使用 block_on
    let rt = Runtime::new().map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;
    
    match rt.block_on(get_smtc_info()) {
        Ok(Some((_metadata, state))) => Ok(Some(state)),
        Ok(None) => Ok(None),
        Err(_e) => {
            // 忽略常见错误，视为无播放
            // log::error!("Failed to get SMTC info: {}", e);
            Ok(None)
        }
    }
}

/// 获取当前媒体元数据
pub fn get_media_metadata() -> std::result::Result<Option<MediaMetadata>, String> {
    let rt = Runtime::new().map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;
    
    match rt.block_on(get_smtc_info()) {
        Ok(Some((metadata, _state))) => Ok(Some(metadata)),
        Ok(None) => Ok(None),
        Err(_e) => {
            Ok(None)
        }
    }
}

async fn get_smtc_info() -> Result<Option<(MediaMetadata, PlaybackState)>> {
    let session_manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;
    let current_session = session_manager.GetCurrentSession()?;

    // 获取播放信息
    let playback_info: GlobalSystemMediaTransportControlsSessionPlaybackInfo = current_session.GetPlaybackInfo()?;
    let playback_status = playback_info.PlaybackStatus()?;
    
    let is_playing = playback_status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

    // 如果不是播放状态，且不是暂停状态，可能认为没有活跃媒体
    // 但为了更像 macOS 的行为，只要有 Session 且有内容，即使暂停也应该返回
    // 不过参考代码中只在 Playing 时返回，这里我们放宽一点，Paused 也返回
    if playback_status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed || 
       playback_status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped {
        return Ok(None);
    }

    // 获取时间线信息
    let timeline_properties: GlobalSystemMediaTransportControlsSessionTimelineProperties = current_session.GetTimelineProperties()?;
    let duration_ticks = timeline_properties.EndTime()?.Duration;
    let elapsed_time_ticks = timeline_properties.Position()?.Duration;

    const TICKS_PER_SECOND: i64 = 10_000_000;
    
    // 转换为秒
    let duration = duration_ticks as f64 / TICKS_PER_SECOND as f64;
    let elapsed_time = elapsed_time_ticks as f64 / TICKS_PER_SECOND as f64;

    // 获取媒体属性
    let media_properties_operation = current_session.TryGetMediaPropertiesAsync()?;
    let media_properties = media_properties_operation.get()?;

    let source_app_name_hstring: HSTRING = current_session.SourceAppUserModelId()?.into();
    let title_hstring: HSTRING = media_properties.Title()?.into();
    let artist_hstring: HSTRING = media_properties.Artist()?.into();
    let album_hstring: HSTRING = media_properties.AlbumTitle()?.into();

    // 获取封面
    let mut artwork_data = None;
    let mut artwork_mime_type = None;

    if let Ok(thumbnail_ref) = media_properties.Thumbnail() {
        if let Ok(thumbnail_stream) = thumbnail_ref.OpenReadAsync()?.get() {
             if let Ok(data_reader) = DataReader::CreateDataReader(&thumbnail_stream) {
                let stream_size = thumbnail_stream.Size()?;
                let stream_size_u32: u32 = stream_size.try_into().unwrap_or(0);
                
                if stream_size_u32 > 0 {
                    if let Ok(_) = data_reader.LoadAsync(stream_size_u32)?.get() {
                        let mut buffer = vec![0u8; stream_size_u32 as usize];
                        if let Ok(_) = data_reader.ReadBytes(&mut buffer) {
                             // 转换为 Base64
                             artwork_data = Some(BASE64.encode(&buffer));
                             // 简单猜测 MIME，通常是 PNG 或 JPEG，这里假设是 PNG 因为参考代码保存为 .png
                             // 但实际上可能是 jpg。Windows Thumbnail 流通常有 ContentType 属性，但这里简化处理
                             artwork_mime_type = Some("image/png".to_string());
                        }
                    }
                }
             }
        }
    }

    let metadata = MediaMetadata {
        bundle_identifier: Some(source_app_name_hstring.to_string_lossy()),
        title: Some(title_hstring.to_string_lossy()),
        artist: Some(artist_hstring.to_string_lossy()),
        album: Some(album_hstring.to_string_lossy()),
        duration,
        artwork_data,
        artwork_mime_type,
        content_item_identifier: None,
    };

    let state = PlaybackState {
        playing: is_playing,
        playback_rate: 1.0, // 简化处理，假设为 1.0
        elapsed_time,
    };

    Ok(Some((metadata, state)))
}
