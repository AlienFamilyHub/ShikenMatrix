use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::Message, Connector};
use futures_util::{SinkExt, StreamExt};
use url::Url;
use tracing::{info, error, warn};

#[cfg(target_os = "windows")]
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use crate::platform::{WindowInfo, MediaMetadata, PlaybackState};

/// Callback types for pushing data to frontend (using usize for thread-safe pointer storage)
pub type LogCallback = Option<extern "C" fn(level: u8, message: *const std::os::raw::c_char, user_data: usize)>;
pub type WindowDataCallback = Option<extern "C" fn(title: *const std::os::raw::c_char, process_name: *const std::os::raw::c_char, pid: u32, icon_data: *const u8, icon_size: usize, user_data: usize)>;
pub type MediaDataCallback = Option<extern "C" fn(title: *const std::os::raw::c_char, artist: *const std::os::raw::c_char, album: *const std::os::raw::c_char, duration: f64, elapsed_time: f64, playing: bool, artwork_data: *const u8, artwork_size: usize, user_data: usize)>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterConfig {
    pub enabled: bool,
    pub ws_url: String,
    pub token: String,
    #[serde(default)]
    pub enable_media_reporting: bool,
}

#[derive(Debug, Clone)]
enum ReporterMessage {
    WindowInfo(WindowInfoMessage),
    MediaPlayback(MediaPlaybackMessage),
    UploadArtwork { content_item_identifier: String, artwork_data: Vec<u8>, mime_type: String },
    Shutdown,
}

#[derive(Debug, Clone, Deserialize)]
struct ServerMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    content_item_identifier: Option<String>,
    #[serde(default)]
    artwork_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct WindowInfoMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: WindowInfoData,
}

#[derive(Debug, Clone, Serialize)]
struct MediaPlaybackMessage {
    #[serde(rename = "type")]
    msg_type: String,
    metadata: MediaMetadataData,
    playback_state: PlaybackStateData,
}

#[derive(Debug, Clone, Serialize)]
struct UploadArtworkMetaMessage {
    #[serde(rename = "type")]
    msg_type: String,
    content_item_identifier: String,
    mime_type: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash)]
struct WindowInfoData {
    title: String,
    process_name: String,
    icon_url: Option<String>,
    app_id: Option<String>,
    pid: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct MediaMetadataData {
    bundle_identifier: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration: f64,
    artwork_url: Option<String>,
    content_item_identifier: Option<String>,
}

impl Hash for MediaMetadataData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bundle_identifier.hash(state);
        self.title.hash(state);
        self.artist.hash(state);
        self.album.hash(state);
        ((self.duration * 1000.0) as i64).hash(state);
        self.artwork_url.hash(state);
        self.content_item_identifier.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct PlaybackStateData {
    playing: bool,
    playback_rate: f64,
    elapsed_time: f64,
}

impl Hash for PlaybackStateData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.playing.hash(state);
        ((self.playback_rate * 100.0) as i64).hash(state);
        (self.elapsed_time as i64).hash(state);
    }
}

fn compute_hash<T: Hash>(data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone)]
pub struct Reporter {
    config: Arc<RwLock<ReporterConfig>>,
    tx: mpsc::UnboundedSender<ReporterMessage>,
    last_window_hash: Arc<AtomicU64>,
    last_media_hash: Arc<AtomicU64>,
    artwork_urls: Arc<RwLock<HashMap<String, String>>>,
    is_connected: Arc<AtomicBool>,
    log_callback: Arc<RwLock<LogCallback>>,
    window_callback: Arc<RwLock<WindowDataCallback>>,
    media_callback: Arc<RwLock<MediaDataCallback>>,
    callback_user_data: Arc<AtomicUsize>,
    is_running: Arc<AtomicBool>,
}

impl Reporter {
    pub fn new(config: ReporterConfig) -> Self {
        let config = Arc::new(RwLock::new(config));
        let artwork_urls = Arc::new(RwLock::new(HashMap::new()));
        let is_connected = Arc::new(AtomicBool::new(false));
        let is_running = Arc::new(AtomicBool::new(true));
        
        let (tx, rx) = mpsc::unbounded_channel();

        let config_clone = config.clone();
        let artwork_urls_clone = artwork_urls.clone();
        let is_connected_clone = is_connected.clone();
        let is_running_clone = is_running.clone();
        
        // Use std::thread to create independent runtime (avoids FFI context issues)
        // Use current_thread runtime to minimize memory usage (saves ~10 threads vs multi_thread)
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");
            rt.block_on(Self::run_reporter(config_clone, rx, artwork_urls_clone, is_connected_clone, is_running_clone));
        });

        let reporter = Self {
            config,
            tx,
            last_window_hash: Arc::new(AtomicU64::new(0)),
            last_media_hash: Arc::new(AtomicU64::new(0)),
            artwork_urls,
            is_connected,
            log_callback: Arc::new(RwLock::new(None)),
            window_callback: Arc::new(RwLock::new(None)),
            media_callback: Arc::new(RwLock::new(None)),
            callback_user_data: Arc::new(AtomicUsize::new(0)),
            is_running,
        };

        // Start window monitoring in a separate thread
        reporter.start_window_monitoring();

        reporter
    }

    /// For FFI: create with external runtime handle
    pub fn new_with_handle(config: ReporterConfig, handle: tokio::runtime::Handle) -> Self {
        let config = Arc::new(RwLock::new(config));
        let artwork_urls = Arc::new(RwLock::new(HashMap::new()));
        let is_connected = Arc::new(AtomicBool::new(false));
        let is_running = Arc::new(AtomicBool::new(true));
        let (tx, rx) = mpsc::unbounded_channel();

        let config_clone = config.clone();
        let artwork_urls_clone = artwork_urls.clone();
        let is_connected_clone = is_connected.clone();
        let is_running_clone = is_running.clone();
        
        handle.spawn(async move {
            Self::run_reporter(config_clone, rx, artwork_urls_clone, is_connected_clone, is_running_clone).await;
        });

        let reporter = Self {
            config,
            tx,
            last_window_hash: Arc::new(AtomicU64::new(0)),
            last_media_hash: Arc::new(AtomicU64::new(0)),
            artwork_urls,
            is_connected,
            log_callback: Arc::new(RwLock::new(None)),
            window_callback: Arc::new(RwLock::new(None)),
            media_callback: Arc::new(RwLock::new(None)),
            callback_user_data: Arc::new(AtomicUsize::new(0)),
            is_running,
        };

        // Start window monitoring in a separate thread
        reporter.start_window_monitoring();

        reporter
    }
    
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        // Send shutdown message to break out of tokio::select! in run_reporter
        let _ = self.tx.send(ReporterMessage::Shutdown);
    }
    
    /// Set callback for logs
    pub fn set_log_callback(&self, callback: LogCallback, user_data: usize) {
        if let Ok(mut cb) = self.log_callback.write() {
            *cb = callback;
        }
        self.callback_user_data.store(user_data, Ordering::Relaxed);
    }
    
    /// Set callback for window data
    pub fn set_window_callback(&self, callback: WindowDataCallback, user_data: usize) {
        if let Ok(mut cb) = self.window_callback.write() {
            *cb = callback;
        }
        self.callback_user_data.store(user_data, Ordering::Relaxed);
    }
    
    /// Set callback for media data
    pub fn set_media_callback(&self, callback: MediaDataCallback, user_data: usize) {
        if let Ok(mut cb) = self.media_callback.write() {
            *cb = callback;
        }
        self.callback_user_data.store(user_data, Ordering::Relaxed);
    }
    
    /// Push log to frontend
    fn push_log(&self, level: u8, message: &str) {
        info!("🔔 push_log called: level={}, message={}", level, message);
        if let Ok(callback) = self.log_callback.read() {
            if let Some(cb) = *callback {
                let user_data = self.callback_user_data.load(Ordering::Relaxed);
                let c_message = std::ffi::CString::new(message).unwrap();
                info!("📤 Calling log callback with user_data={}", user_data);
                cb(level, c_message.as_ptr(), user_data);
            } else {
                info!("⚠️ Log callback is None");
            }
        }
    }
    
    /// Push window data to frontend
    fn push_window_data(&self, title: &str, process_name: &str, pid: u32, icon_data: Option<&[u8]>) {
        info!("🔔 push_window_data called: title={}, process={}, pid={}, icon={}", 
              title, process_name, pid, icon_data.map(|d| d.len()).unwrap_or(0));
        if let Ok(callback) = self.window_callback.read() {
            if let Some(cb) = *callback {
                let user_data = self.callback_user_data.load(Ordering::Relaxed);
                let c_title = std::ffi::CString::new(title).unwrap();
                let c_process = std::ffi::CString::new(process_name).unwrap();
                
                // Don't send icon data to frontend to save memory/copy overhead
                // Frontend can fetch icon directly using PID if needed
                let (icon_ptr, icon_len) = (std::ptr::null(), 0);
                
                info!("📤 Calling window callback with user_data={}", user_data);
                cb(c_title.as_ptr(), c_process.as_ptr(), pid, icon_ptr, icon_len, user_data);
            } else {
                info!("⚠️ Window callback is None");
            }
        }
    }
    
    /// Push media data to frontend
    fn push_media_data(&self, title: &str, artist: &str, album: &str, duration: f64, elapsed_time: f64, playing: bool, artwork_data: Option<&[u8]>) {
        info!("🔔 push_media_data called: title={}, artist={}, artwork={}", 
              title, artist, artwork_data.map(|d| d.len()).unwrap_or(0));
        if let Ok(callback) = self.media_callback.read() {
            if let Some(cb) = *callback {
                let user_data = self.callback_user_data.load(Ordering::Relaxed);
                let c_title = std::ffi::CString::new(title).unwrap_or_default();
                let c_artist = std::ffi::CString::new(artist).unwrap_or_default();
                let c_album = std::ffi::CString::new(album).unwrap_or_default();
                
                let (artwork_ptr, artwork_len) = if let Some(data) = artwork_data {
                    (data.as_ptr(), data.len())
                } else {
                    (std::ptr::null(), 0)
                };
                
                info!("📤 Calling media callback with user_data={}", user_data);
                cb(c_title.as_ptr(), c_artist.as_ptr(), c_album.as_ptr(), duration, elapsed_time, playing, artwork_ptr, artwork_len, user_data);
            } else {
                info!("⚠️ Media callback is None");
            }
        }
    }

    /// Start monitoring window changes in a background thread
    fn start_window_monitoring(&self) {
        let reporter_clone = self.clone();
        
        std::thread::spawn(move || {
            // Wait a bit for callbacks to be registered from frontend
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            reporter_clone.push_log(0, "窗口监控已启动");
            let mut permission_warned = false;
            let mut check_count = 0;
            
            // Allow comparison of Option<T>
            let mut last_window_info: Option<crate::platform::WindowInfo> = None;
            let mut last_media_metadata: Option<crate::platform::MediaMetadata> = None;
            let mut last_playback_state: Option<crate::platform::PlaybackState> = None;
            
            loop {
                // Check running status
                if !reporter_clone.is_running.load(Ordering::Relaxed) {
                    reporter_clone.push_log(0, "监控线程正在退出...");
                    break;
                }
                
                std::thread::sleep(std::time::Duration::from_secs(1));
                check_count += 1;
                
                // Check if reporter is enabled
                let enabled = reporter_clone.config.read()
                    .map(|cfg| cfg.enabled)
                    .unwrap_or(false);
                
                if !enabled {
                    if check_count % 10 == 0 {
                        reporter_clone.push_log(0, &format!("窗口监控: reporter 已禁用，跳过检查 #{}", check_count));
                    }
                    continue; // Skip monitoring if disabled
                }
                
                #[cfg(target_os = "macos")]
                {
                    objc2::rc::autoreleasepool(|_| {
                        // Monitor window info
                        match crate::platform::macos::get_frontmost_window_info_sync() {
                            Ok(window_info) => {
                                if last_window_info.as_ref() != Some(&window_info) {
                                    let log_msg = format!("获取到窗口信息: {} ({})", window_info.title, window_info.process_name);
                                    reporter_clone.push_log(0, &log_msg);
                                    
                                    // Push window data to frontend (with icon if available)
                                    reporter_clone.push_window_data(
                                        &window_info.title, 
                                        &window_info.process_name, 
                                        window_info.pid as u32,
                                        window_info.icon_data.as_deref()
                                    );
                                    
                                    reporter_clone.send_window_info(&window_info);
                                    permission_warned = false; // Reset warning flag on success
                                    
                                    last_window_info = Some(window_info);
                                }
                            }
                            Err(e) => {
                                if !permission_warned {
                                    let err_msg = format!("获取窗口信息失败: {}", e);
                                    reporter_clone.push_log(1, &err_msg);
                                    permission_warned = true; // Only warn once
                                }
                            }
                        }
                    
                        // Monitor media playback (every second)
                        // DISABLED by default - set ENABLE_MEDIA_REPORTING=1 to enable
                        if std::env::var("ENABLE_MEDIA_REPORTING").unwrap_or_default() == "1" {
                            if let Ok(Some(metadata)) = crate::platform::macos::get_media_metadata() {
                                if let Ok(Some(state)) = crate::platform::macos::get_playback_state() {
                                    
                                    let metadata_changed = last_media_metadata.as_ref() != Some(&metadata);
                                    let state_changed = last_playback_state.as_ref() != Some(&state);

                                    if metadata_changed || state_changed {
                                        // Get artwork slice directly from Arc (no decoding needed)
                                        let artwork_slice = metadata.artwork_data.as_deref().map(|v| v.as_slice());
                                        
                                        // Push media data to frontend
                                        let title = metadata.title.as_deref().unwrap_or("未知");
                                        let artist = metadata.artist.as_deref().unwrap_or("未知");
                                        let album = metadata.album.as_deref().unwrap_or("未知");
                                        reporter_clone.push_media_data(
                                            title, 
                                            artist, 
                                            album, 
                                            metadata.duration, 
                                            state.elapsed_time, 
                                            state.playing,
                                            artwork_slice
                                        );
                                        
                                        reporter_clone.send_media_playback(&metadata, &state);

                                        // Upload artwork if available and not cached (only if metadata changed)
                                        if metadata_changed {
                                            if let (Some(artwork_data), Some(mime_type), Some(content_id)) =
                                                (metadata.artwork_data.as_ref(), metadata.artwork_mime_type.as_ref(), metadata.content_item_identifier.as_ref()) {
                                                // Check if already cached
                                                let needs_upload = reporter_clone.artwork_urls.read()
                                                    .map(|urls| !urls.contains_key(content_id))
                                                    .unwrap_or(true);

                                                if needs_upload {
                                                    reporter_clone.upload_artwork(content_id.clone(), artwork_data.to_vec(), mime_type.clone());
                                                }
                                            }
                                        }
                                        
                                        last_media_metadata = Some(metadata);
                                        last_playback_state = Some(state);
                                    }
                                }
                            }
                        }
                    });
                }
                
                #[cfg(target_os = "windows")]
                {
                    // Monitor window info
                    match crate::platform::windows::get_frontmost_window() {
                        Ok(window_info) => {
                            if last_window_info.as_ref() != Some(&window_info) {
                                let log_msg = format!("获取到窗口信息: {} ({})", window_info.title, window_info.process_name);
                                reporter_clone.push_log(0, &log_msg);
                                
                                // Push window data to frontend (with icon if available)
                                reporter_clone.push_window_data(
                                    &window_info.title, 
                                    &window_info.process_name, 
                                    window_info.pid as u32, 
                                    window_info.icon_data.as_ref().map(|v| v.as_slice())
                                );
                                
                                reporter_clone.send_window_info(&window_info);
                                permission_warned = false; // Reset warning flag on success
                                
                                last_window_info = Some(window_info);
                            }
                        }
                        Err(e) => {
                            if !permission_warned {
                                let err_msg = format!("获取窗口信息失败: {}", e);
                                reporter_clone.push_log(1, &err_msg);
                                permission_warned = true;
                            }
                        }
                    }

                    // Monitor media playback (every second)
                    // DISABLED by default - set ENABLE_MEDIA_REPORTING=1 to enable
                    if std::env::var("ENABLE_MEDIA_REPORTING").unwrap_or_default() == "1" {
                        if let Ok(Some(metadata)) = crate::platform::windows::get_media_metadata() {
                            if let Ok(Some(state)) = crate::platform::windows::get_playback_state() {

                                let metadata_changed = last_media_metadata.as_ref() != Some(&metadata);
                                let state_changed = last_playback_state.as_ref() != Some(&state);

                                if metadata_changed || state_changed {
                                    // Decode Base64 artwork if available
                                    let artwork_vec = metadata.artwork_data.as_ref().and_then(|b64| {
                                        BASE64.decode(b64).ok()
                                    });
                                    let artwork_slice = artwork_vec.as_ref().map(|v: &Vec<u8>| v.as_slice());

                                    // Push media data to frontend
                                    let title = metadata.title.as_deref().unwrap_or("未知");
                                    let artist = metadata.artist.as_deref().unwrap_or("未知");
                                    let album = metadata.album.as_deref().unwrap_or("未知");
                                    reporter_clone.push_media_data(
                                        title,
                                        artist,
                                        album,
                                        metadata.duration,
                                        state.elapsed_time,
                                        state.playing,
                                        artwork_slice.as_deref()
                                    );

                                    // Note: Windows doesn't send_media_playback or upload_artwork
                                    // due to platform differences in media API structure

                                    last_media_metadata = Some(metadata);
                                    last_playback_state = Some(state);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }

    async fn run_reporter(
        config: Arc<RwLock<ReporterConfig>>,
        mut rx: mpsc::UnboundedReceiver<ReporterMessage>,
        artwork_urls: Arc<RwLock<HashMap<String, String>>>,
        is_connected: Arc<AtomicBool>,
        is_running: Arc<AtomicBool>,
    ) {
        let mut reconnect_attempts = 0;
        const MAX_RECONNECT_ATTEMPTS: u32 = 5;
        const RECONNECT_INTERVAL: u64 = 3000;

        loop {
            // Check if stop requested
            if !is_running.load(Ordering::Relaxed) {
                info!("Reporter stopping (run_reporter loop)");
                break;
            }

            let cfg = config.read().unwrap().clone();

            if !cfg.enabled {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }

            let ws_url_str = cfg.ws_url
                .replace("http://", "ws://")
                .replace("https://", "wss://");

            let ws_url = match Url::parse(&ws_url_str) {
                Ok(mut url) => {
                    url.query_pairs_mut().append_pair("token", &cfg.token);
                    url
                }
                Err(e) => {
                    error!("Invalid WebSocket URL: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            info!("Connecting to WebSocket: {}", ws_url);
            is_connected.store(false, Ordering::Relaxed);

            // Create TLS connector that forces HTTP/1.1 (required for WebSocket over HTTPS)
            let connector = Connector::Rustls(Arc::new(
                rustls::ClientConfig::builder()
                    .with_root_certificates(rustls::RootCertStore::from_iter(
                        webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
                    ))
                    .with_no_client_auth()
            ));

            let connect_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(15),
                connect_async_tls_with_config(ws_url.as_str(), None, false, Some(connector))
            ).await;

            match connect_result {
                Ok(Ok((ws_stream, response))) => {
                    info!("✅ WebSocket connected! Status: {}", response.status());
                    is_connected.store(true, Ordering::Relaxed);
                    reconnect_attempts = 0;

                    let (mut write, mut read) = ws_stream.split();
                    let is_connected_clone_inner = is_connected.clone();

                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                match msg {
                                    ReporterMessage::WindowInfo(window_msg) => {
                                        if let Ok(json) = serde_json::to_string(&window_msg) {
                                            if let Err(e) = write.send(Message::Text(json.into())).await {
                                                error!("Failed to send window message: {}", e);
                                                break;
                                            }
                                        }
                                    }
                                    ReporterMessage::MediaPlayback(media_msg) => {
                                        if let Ok(json) = serde_json::to_string(&media_msg) {
                                            if let Err(e) = write.send(Message::Text(json.into())).await {
                                                error!("Failed to send media message: {}", e);
                                                break;
                                            }
                                        }
                                    }
                                    ReporterMessage::UploadArtwork { content_item_identifier, artwork_data, mime_type } => {
                                        let meta_msg = UploadArtworkMetaMessage {
                                            msg_type: "upload_artwork_meta".to_string(),
                                            content_item_identifier: content_item_identifier.clone(),
                                            mime_type,
                                        };
                                        if let Ok(meta_json) = serde_json::to_string(&meta_msg) {
                                            if write.send(Message::Text(meta_json.into())).await.is_ok() {
                                                if let Err(e) = write.send(Message::Binary(artwork_data.into())).await {
                                                    error!("Failed to send artwork: {}", e);
                                                    break;
                                                }
                                                info!("Artwork uploaded: {}", content_item_identifier);
                                            }
                                        }
                                    }
                                    ReporterMessage::Shutdown => {
                                        info!("Reporter shutdown signal received in run loop");
                                        is_connected_clone_inner.store(false, Ordering::Relaxed);
                                        // Close WebSocket connection cleanly if possible
                                        let _ = write.close().await;
                                        return; // Exit the run_reporter function completely
                                    }
                                }
                            }
                            Some(msg) = read.next() => {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        info!("Received: {}", text);
                                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                                            if server_msg.msg_type == "artwork_uploaded" {
                                                if let (Some(content_id), Some(url)) = (server_msg.content_item_identifier, server_msg.artwork_url) {
                                                    if let Ok(mut urls) = artwork_urls.write() {
                                                        urls.insert(content_id, url);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        warn!("WebSocket closed by server");
                                        break;
                                    }
                                    Err(e) => {
                                        error!("WebSocket error: {}", e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    is_connected.store(false, Ordering::Relaxed);
                }
                Ok(Err(e)) => {
                    error!("❌ WebSocket connection failed: {}", e);
                    is_connected.store(false, Ordering::Relaxed);
                }
                Err(_) => {
                    error!("❌ WebSocket connection timeout (15s)");
                    is_connected.store(false, Ordering::Relaxed);
                }
            }

            reconnect_attempts += 1;
            if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
                error!("Max reconnect attempts reached, waiting 30s");
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                reconnect_attempts = 0;
            } else {
                info!("Reconnecting {}/{}...", reconnect_attempts, MAX_RECONNECT_ATTEMPTS);
                tokio::time::sleep(tokio::time::Duration::from_millis(RECONNECT_INTERVAL)).await;
            }
        }
    }

    #[allow(dead_code)]
    pub fn update_config(&self, config: ReporterConfig) {
        if let Ok(mut cfg) = self.config.write() {
            *cfg = config;
            info!("Configuration updated");
        }
    }

    pub fn send_window_info(&self, info: &WindowInfo) {
        let data = WindowInfoData {
            title: info.title.clone(),
            process_name: info.process_name.clone(),
            icon_url: None,
            app_id: info.app_id.clone(),
            pid: info.pid as u32,
        };

        let new_hash = compute_hash(&data);
        let old_hash = self.last_window_hash.swap(new_hash, Ordering::Relaxed);

        if new_hash != old_hash {
            let log_msg = format!("📤 发送窗口信息: {} ({})", data.title, data.process_name);
            self.push_log(0, &log_msg);
            
            let msg = ReporterMessage::WindowInfo(WindowInfoMessage {
                msg_type: "window_info".to_string(),
                data,
            });
            if let Err(e) = self.tx.send(msg) {
                let err_msg = format!("发送窗口信息到通道失败: {}", e);
                self.push_log(2, &err_msg);
            }
        } else {
            // Window hasn't changed, skip sending
        }
    }

    pub fn send_media_playback(&self, metadata: &MediaMetadata, state: &PlaybackState) {
        let artwork_url = metadata.content_item_identifier.as_ref()
            .and_then(|id| self.artwork_urls.read().ok()?.get(id).cloned());

        let metadata_data = MediaMetadataData {
            bundle_identifier: metadata.bundle_identifier.clone(),
            title: metadata.title.clone(),
            artist: metadata.artist.clone(),
            album: metadata.album.clone(),
            duration: metadata.duration,
            artwork_url,
            content_item_identifier: metadata.content_item_identifier.clone(),
        };

        let state_data = PlaybackStateData {
            playing: state.playing,
            playback_rate: state.playback_rate,
            elapsed_time: state.elapsed_time,
        };

        let new_hash = compute_hash(&(&metadata_data, &state_data));
        let old_hash = self.last_media_hash.swap(new_hash, Ordering::Relaxed);

        if new_hash != old_hash {
            let msg = ReporterMessage::MediaPlayback(MediaPlaybackMessage {
                msg_type: "media_playback".to_string(),
                metadata: metadata_data,
                playback_state: state_data,
            });
            let _ = self.tx.send(msg);
        }
    }

    pub fn upload_artwork(&self, content_item_identifier: String, artwork_data: Vec<u8>, mime_type: String) {
        let _ = self.tx.send(ReporterMessage::UploadArtwork {
            content_item_identifier,
            artwork_data,
            mime_type,
        });
    }
}
