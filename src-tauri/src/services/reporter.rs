use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use url::Url;

use crate::platform::{WindowInfo, MediaMetadata, PlaybackState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterConfig {
    pub enabled: bool,
    pub ws_url: String,
    pub token: String,
}

#[derive(Debug, Clone)]
enum ReporterMessage {
    WindowInfo(WindowInfoMessage),
    MediaPlayback(MediaPlaybackMessage),
    UploadArtwork { content_item_identifier: String, artwork_data: Vec<u8>, mime_type: String },
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

pub struct Reporter {
    config: Arc<RwLock<ReporterConfig>>,
    tx: mpsc::UnboundedSender<ReporterMessage>,
    last_window_hash: AtomicU64,
    last_media_hash: AtomicU64,
    artwork_urls: Arc<RwLock<HashMap<String, String>>>,
}

impl Reporter {
    pub fn new(config: ReporterConfig) -> Self {
        let config = Arc::new(RwLock::new(config));
        let artwork_urls = Arc::new(RwLock::new(HashMap::new()));
        let (tx, rx) = mpsc::unbounded_channel();

        let config_clone = config.clone();
        let artwork_urls_clone = artwork_urls.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(Self::run_reporter(config_clone, rx, artwork_urls_clone));
        });

        Self {
            config,
            tx,
            last_window_hash: AtomicU64::new(0),
            last_media_hash: AtomicU64::new(0),
            artwork_urls,
        }
    }

    async fn run_reporter(
        config: Arc<RwLock<ReporterConfig>>,
        mut rx: mpsc::UnboundedReceiver<ReporterMessage>,
        artwork_urls: Arc<RwLock<HashMap<String, String>>>,
    ) {
        let mut reconnect_attempts = 0;
        const MAX_RECONNECT_ATTEMPTS: u32 = 5;
        const RECONNECT_INTERVAL: u64 = 3000;

        loop {
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
                    eprintln!("❌ 无效的 WebSocket URL: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            println!("🔄 正在连接到 WebSocket: {}", ws_url);

            match connect_async(ws_url.as_str()).await {
                Ok((ws_stream, _)) => {
                    println!("✅ WebSocket 连接成功");
                    reconnect_attempts = 0;

                    let (mut write, mut read) = ws_stream.split();

                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                match msg {
                                    ReporterMessage::WindowInfo(window_msg) => {
                                        let json = match serde_json::to_string(&window_msg) {
                                            Ok(j) => j,
                                            Err(e) => {
                                                eprintln!("❌ 序列化窗口消息失败: {}", e);
                                                continue;
                                            }
                                        };
                                        if let Err(e) = write.send(Message::Text(json)).await {
                                            eprintln!("❌ 发送窗口消息失败: {}", e);
                                            break;
                                        }
                                    }
                                    ReporterMessage::MediaPlayback(media_msg) => {
                                        let json = match serde_json::to_string(&media_msg) {
                                            Ok(j) => j,
                                            Err(e) => {
                                                eprintln!("❌ 序列化媒体消息失败: {}", e);
                                                continue;
                                            }
                                        };
                                        if let Err(e) = write.send(Message::Text(json)).await {
                                            eprintln!("❌ 发送媒体消息失败: {}", e);
                                            break;
                                        }
                                    }
                                    ReporterMessage::UploadArtwork { content_item_identifier, artwork_data, mime_type } => {
                                        // 步骤 1: 发送元数据
                                        let meta_msg = UploadArtworkMetaMessage {
                                            msg_type: "upload_artwork_meta".to_string(),
                                            content_item_identifier: content_item_identifier.clone(),
                                            mime_type,
                                        };
                                        let meta_json = match serde_json::to_string(&meta_msg) {
                                            Ok(j) => j,
                                            Err(e) => {
                                                eprintln!("❌ 序列化封面元数据失败: {}", e);
                                                continue;
                                            }
                                        };
                                        if let Err(e) = write.send(Message::Text(meta_json)).await {
                                            eprintln!("❌ 发送封面元数据失败: {}", e);
                                            break;
                                        }
                                        
                                        // 步骤 2: 发送二进制数据
                                        if let Err(e) = write.send(Message::Binary(artwork_data)).await {
                                            eprintln!("❌ 发送封面二进制数据失败: {}", e);
                                            break;
                                        }
                                        println!("📤 封面已上传: {}", content_item_identifier);
                                    }
                                }
                            }
                            Some(msg) = read.next() => {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        println!("📨 收到消息: {}", text);
                                        // 处理服务器响应
                                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                                            if server_msg.msg_type == "artwork_uploaded" {
                                                if let (Some(content_id), Some(url)) = (server_msg.content_item_identifier, server_msg.artwork_url) {
                                                    if let Ok(mut urls) = artwork_urls.write() {
                                                        urls.insert(content_id.clone(), url.clone());
                                                        println!("🖼️ 封面 URL 已缓存: {} -> {}", content_id, url);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        println!("❌ WebSocket 连接已关闭");
                                        break;
                                    }
                                    Err(e) => {
                                        eprintln!("❌ 接收消息失败: {}", e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ WebSocket 连接失败: {}", e);
                    reconnect_attempts += 1;

                    if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
                        eprintln!("❌ 达到最大重连次数，停止重连");
                        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                        reconnect_attempts = 0;
                    } else {
                        println!("🔄 {}/{} 次重连尝试...", reconnect_attempts, MAX_RECONNECT_ATTEMPTS);
                        tokio::time::sleep(tokio::time::Duration::from_millis(RECONNECT_INTERVAL)).await;
                    }
                }
            }
        }
    }

    pub fn update_config(&self, config: ReporterConfig) {
        if let Ok(mut cfg) = self.config.write() {
            *cfg = config;
            println!("🔄 配置已更新");
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
            let msg = ReporterMessage::WindowInfo(WindowInfoMessage {
                msg_type: "window_info".to_string(),
                data,
            });
            if let Err(e) = self.tx.send(msg) {
                eprintln!("❌ 发送窗口信息失败: {}", e);
            } else {
                println!("📤 窗口信息已变化，已发送");
            }
        }
    }

    pub fn send_media_playback(&self, metadata: &MediaMetadata, state: &PlaybackState) {
        // 从缓存中查找封面 URL
        let artwork_url = if let Some(ref content_id) = metadata.content_item_identifier {
            self.artwork_urls.read().ok().and_then(|urls| urls.get(content_id).cloned())
        } else {
            None
        };

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

        let combined = (&metadata_data, &state_data);
        let new_hash = compute_hash(&combined);
        let old_hash = self.last_media_hash.swap(new_hash, Ordering::Relaxed);

        if new_hash != old_hash {
            let msg = ReporterMessage::MediaPlayback(MediaPlaybackMessage {
                msg_type: "media_playback".to_string(),
                metadata: metadata_data,
                playback_state: state_data,
            });

            if let Err(e) = self.tx.send(msg) {
                eprintln!("❌ 发送媒体播放状态失败: {}", e);
            } else {
                println!("📤 媒体播放状态已变化，已发送");
            }
        }
    }

    pub fn upload_artwork(&self, content_item_identifier: String, artwork_data: Vec<u8>, mime_type: String) {
        let msg = ReporterMessage::UploadArtwork {
            content_item_identifier,
            artwork_data,
            mime_type,
        };

        if let Err(e) = self.tx.send(msg) {
            eprintln!("❌ 发送封面上传请求失败: {}", e);
        } else {
            println!("📤 封面上传请求已发送");
        }
    }
}
