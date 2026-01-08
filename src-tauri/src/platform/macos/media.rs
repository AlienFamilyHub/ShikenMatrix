//! macOS 媒体播放信息获取模块
//! 使用 mediaremote-rs 库访问 MediaRemote.framework

use mediaremote_rs::{get_now_playing, is_playing, NowPlayingInfo};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};

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
    /// 应用 Bundle ID
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

/// 媒体信息缓存
struct MediaCache {
    metadata: Option<MediaMetadata>,
    playback_state: Option<PlaybackState>,
    last_update: Instant,
    /// 用于判断封面是否变化的标识
    artwork_key: Option<String>,
}

impl Default for MediaCache {
    fn default() -> Self {
        Self {
            metadata: None,
            playback_state: None,
            last_update: Instant::now() - Duration::from_secs(10), // 确保首次会更新
            artwork_key: None,
        }
    }
}

// 全局缓存，缓存时间 200ms（避免频繁调用系统 API）
static MEDIA_CACHE: Mutex<Option<MediaCache>> = Mutex::new(None);
const CACHE_DURATION_MS: u64 = 200;

/// 从 NowPlayingInfo 提取数据并更新缓存
fn update_cache_from_info(info: &NowPlayingInfo, cache: &mut MediaCache) {
    // 生成封面标识（用于判断封面是否变化）
    let new_artwork_key = format!(
        "{}:{}:{}",
        info.bundle_identifier,
        info.title,
        info.album.as_deref().unwrap_or("")
    );
    
    let key_changed = cache.artwork_key.as_ref() != Some(&new_artwork_key);
    
    // 封面数据更新逻辑：
    // 1. 如果有新的封面数据，始终使用新数据
    // 2. 如果歌曲没变（key 相同），复用缓存的封面
    // 3. 如果歌曲变了但新封面为空，清空封面（避免显示旧歌曲的封面）
    let (artwork_data, artwork_mime_type) = if info.artwork_data.is_some() {
        // 有新的封面数据，更新 key 并使用新数据
        cache.artwork_key = Some(new_artwork_key);
        (info.artwork_data.clone(), info.artwork_mime_type.clone())
    } else if !key_changed {
        // 歌曲没变，复用缓存的封面
        let cached_artwork = cache.metadata.as_ref().and_then(|m| m.artwork_data.clone());
        let cached_mime = cache.metadata.as_ref().and_then(|m| m.artwork_mime_type.clone());
        (cached_artwork, cached_mime)
    } else {
        // 歌曲变了但没有封面数据，更新 key 并清空封面
        cache.artwork_key = Some(new_artwork_key);
        (None, None)
    };

    cache.metadata = Some(MediaMetadata {
        bundle_identifier: if info.bundle_identifier.is_empty() {
            None
        } else {
            Some(info.bundle_identifier.clone())
        },
        title: if info.title.is_empty() {
            None
        } else {
            Some(info.title.clone())
        },
        artist: info.artist.clone(),
        album: info.album.clone(),
        duration: info.duration.unwrap_or(0.0),
        artwork_data,
        artwork_mime_type,
        // 生成内容标识符：使用 bundle_id + title + album 的组合
        content_item_identifier: Some(format!(
            "{}:{}:{}",
            info.bundle_identifier,
            info.title,
            info.album.as_deref().unwrap_or("")
        )),
    });

    cache.playback_state = Some(PlaybackState {
        playing: info.playing,
        playback_rate: info.playback_rate.unwrap_or(if info.playing { 1.0 } else { 0.0 }),
        elapsed_time: info.elapsed_time.unwrap_or(0.0),
    });

    cache.last_update = Instant::now();
}

/// 检查缓存是否有效
fn is_cache_valid(cache: &MediaCache) -> bool {
    cache.last_update.elapsed() < Duration::from_millis(CACHE_DURATION_MS)
}

/// 获取当前播放状态
pub fn get_playback_state() -> Result<Option<PlaybackState>, String> {
    // 尝试从缓存获取
    {
        let cache_guard = MEDIA_CACHE.lock().map_err(|e| format!("缓存锁定失败: {}", e))?;
        if let Some(ref cache) = *cache_guard {
            if is_cache_valid(cache) {
                return Ok(cache.playback_state.clone());
            }
        }
    }

    // 缓存失效，重新获取
    match get_now_playing() {
        Some(info) => {
            let mut cache_guard = MEDIA_CACHE.lock().map_err(|e| format!("缓存锁定失败: {}", e))?;
            let cache = cache_guard.get_or_insert_with(MediaCache::default);
            update_cache_from_info(&info, cache);
            Ok(cache.playback_state.clone())
        },
        None => {
            // 清空缓存
            if let Ok(mut cache_guard) = MEDIA_CACHE.lock() {
                if let Some(ref mut cache) = *cache_guard {
                    cache.metadata = None;
                    cache.playback_state = None;
                    cache.last_update = Instant::now();
                }
            }
            Ok(None)
        },
    }
}

/// 获取当前媒体元数据
pub fn get_media_metadata() -> Result<Option<MediaMetadata>, String> {
    // 尝试从缓存获取
    {
        let cache_guard = MEDIA_CACHE.lock().map_err(|e| format!("缓存锁定失败: {}", e))?;
        if let Some(ref cache) = *cache_guard {
            if is_cache_valid(cache) {
                return Ok(cache.metadata.clone());
            }
        }
    }

    // 缓存失效，重新获取
    match get_now_playing() {
        Some(info) => {
            let mut cache_guard = MEDIA_CACHE.lock().map_err(|e| format!("缓存锁定失败: {}", e))?;
            let cache = cache_guard.get_or_insert_with(MediaCache::default);
            update_cache_from_info(&info, cache);
            Ok(cache.metadata.clone())
        },
        None => {
            // 清空缓存
            if let Ok(mut cache_guard) = MEDIA_CACHE.lock() {
                if let Some(ref mut cache) = *cache_guard {
                    cache.metadata = None;
                    cache.playback_state = None;
                    cache.last_update = Instant::now();
                }
            }
            Ok(None)
        },
    }
}

/// 检查是否有媒体正在播放
#[allow(unused)]
pub fn check_is_playing() -> bool {
    is_playing()
}

/// 获取完整的 NowPlayingInfo (直接返回 mediaremote-rs 的结构)
#[allow(unused)]
pub fn get_now_playing_info() -> Option<NowPlayingInfo> {
    get_now_playing()
}
