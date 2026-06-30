mod monitor;
mod types;
mod websocket;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock, mpsc};
use tokio::sync::mpsc as tokio_mpsc;
use tracing::{error, info, warn};

use crate::platform::{MediaMetadata, PlaybackState, WindowInfo};

pub use types::{LogLevel, ReporterConfig, ReporterEvent};
use types::{
    MediaMetadataData, MediaPlaybackMessage, PlaybackStateData, ReporterMessage, WindowInfoData,
    WindowInfoMessage, compute_hash,
};

// ---------------------------------------------------------------------------
// Monitor — platform polling + frontend event emission
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Monitor {
    last_window_hash: Arc<AtomicU64>,
    last_media_hash: Arc<AtomicU64>,
    is_running: Arc<AtomicBool>,
    event_tx: Option<mpsc::Sender<ReporterEvent>>,
    reporter_tx: Arc<RwLock<Option<tokio_mpsc::UnboundedSender<ReporterMessage>>>>,
    artwork_urls: Arc<RwLock<HashMap<String, String>>>,
}

impl Monitor {
    pub fn new(event_tx: Option<mpsc::Sender<ReporterEvent>>) -> Self {
        let monitor = Self {
            last_window_hash: Arc::new(AtomicU64::new(0)),
            last_media_hash: Arc::new(AtomicU64::new(0)),
            is_running: Arc::new(AtomicBool::new(true)),
            event_tx,
            reporter_tx: Arc::new(RwLock::new(None)),
            artwork_urls: Arc::new(RwLock::new(HashMap::new())),
        };

        monitor.start_monitoring();
        monitor
    }

    pub fn artwork_urls(&self) -> Arc<RwLock<HashMap<String, String>>> {
        self.artwork_urls.clone()
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        // Detach any reporter
        self.detach_reporter();
        self.emit_log(LogLevel::Info, "监控已停止");
    }

    /// Attach a Reporter — monitor will forward data messages to it without exposing the channel type.
    pub fn attach_reporter(&self, reporter: &Reporter) {
        if let Ok(mut guard) = self.reporter_tx.write() {
            *guard = Some(reporter.tx.clone());
        }
    }

    /// Detach the Reporter — stop forwarding data, send Shutdown.
    pub fn detach_reporter(&self) {
        if let Ok(mut guard) = self.reporter_tx.write() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(ReporterMessage::Shutdown);
            }
        }
    }

    pub fn has_reporter(&self) -> bool {
        self.reporter_tx
            .read()
            .map(|g| g.is_some())
            .unwrap_or(false)
    }

    pub(super) fn emit_log(&self, level: LogLevel, message: impl Into<String>) {
        let message = message.into();
        match level {
            LogLevel::Info => info!("{message}"),
            LogLevel::Warn => warn!("{message}"),
            LogLevel::Error => error!("{message}"),
        }

        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.send(ReporterEvent::Log { level, message });
        }
    }

    pub(super) fn emit_window(&self, info: &WindowInfo) {
        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.send(ReporterEvent::WindowUpdated {
                title: info.title.clone(),
                process_name: info.process_name.clone(),
                pid: info.pid as u32,
                icon_data: info.icon_data.clone(),
            });
        }
    }

    pub(super) fn emit_media(
        &self,
        metadata: &MediaMetadata,
        state: &PlaybackState,
        artwork_data: Option<Vec<u8>>,
    ) {
        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.send(ReporterEvent::MediaUpdated {
                title: metadata.title.clone().unwrap_or_else(|| "未知".to_string()),
                artist: metadata
                    .artist
                    .clone()
                    .unwrap_or_else(|| "未知".to_string()),
                album: metadata.album.clone().unwrap_or_else(|| "未知".to_string()),
                duration: metadata.duration,
                elapsed_time: state.elapsed_time,
                playing: state.playing,
                artwork_data,
            });
        }
    }

    pub fn send_window_info(&self, info: &WindowInfo) {
        let data = WindowInfoData {
            title: info.title.clone(),
            process_name: info.process_name.clone(),
            icon_base64: info.icon_data.as_ref().map(|data| BASE64.encode(data)),
            icon_url: None,
            app_id: info.app_id.clone(),
            pid: info.pid as u32,
        };

        let new_hash = compute_hash(&data);
        let old_hash = self.last_window_hash.swap(new_hash, Ordering::Relaxed);

        if new_hash != old_hash {
            let message = ReporterMessage::WindowInfo(WindowInfoMessage {
                msg_type: "window_info".to_string(),
                data,
            });
            if let Ok(guard) = self.reporter_tx.read() {
                if let Some(tx) = guard.as_ref() {
                    if let Err(error) = tx.send(message) {
                        self.emit_log(LogLevel::Error, format!("发送窗口信息到通道失败: {error}"));
                    }
                }
            }
        }
    }

    pub fn send_media_playback(&self, metadata: &MediaMetadata, state: &PlaybackState) {
        let artwork_url = metadata
            .content_item_identifier
            .as_ref()
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
            if let Ok(guard) = self.reporter_tx.read() {
                if let Some(tx) = guard.as_ref() {
                    let _ = tx.send(ReporterMessage::MediaPlayback(MediaPlaybackMessage {
                        msg_type: "media_playback".to_string(),
                        metadata: metadata_data,
                        playback_state: state_data,
                    }));
                }
            }
        }
    }

    pub fn upload_artwork(
        &self,
        content_item_identifier: String,
        artwork_data: Vec<u8>,
        mime_type: String,
    ) {
        if let Ok(guard) = self.reporter_tx.read() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(ReporterMessage::UploadArtwork {
                    content_item_identifier,
                    artwork_data,
                    mime_type,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Reporter — WebSocket connection & data reporting
// ---------------------------------------------------------------------------

pub struct Reporter {
    is_connected: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    last_error: Arc<RwLock<Option<String>>>,
    tx: tokio_mpsc::UnboundedSender<ReporterMessage>,
}

impl Reporter {
    /// Create and start a new Reporter.
    /// The caller must attach it to a Monitor via `monitor.attach_reporter(&reporter)`.
    pub fn new(config: ReporterConfig, artwork_urls: Arc<RwLock<HashMap<String, String>>>) -> Self {
        let is_connected = Arc::new(AtomicBool::new(false));
        let is_running = Arc::new(AtomicBool::new(true));
        let last_error: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
        let (tx, rx) = tokio_mpsc::unbounded_channel();

        let config = Arc::new(RwLock::new(config));

        {
            let config = config.clone();
            let artwork_urls = artwork_urls.clone();
            let is_connected = is_connected.clone();
            let is_running = is_running.clone();
            let last_error = last_error.clone();

            std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime");
                runtime.block_on(async move {
                    Self::run_reporter(
                        config,
                        rx,
                        artwork_urls,
                        is_connected,
                        is_running,
                        last_error,
                    )
                    .await;
                });
            });
        }

        Self {
            is_connected,
            is_running,
            last_error,
            tx,
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        let _ = self.tx.send(ReporterMessage::Shutdown);
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error.read().ok().and_then(|error| error.clone())
    }

    pub(super) fn set_last_error(
        last_error: &Arc<RwLock<Option<String>>>,
        message: impl Into<String>,
    ) -> String {
        let message = message.into();
        if let Ok(mut error) = last_error.write() {
            *error = Some(message.clone());
        }
        message
    }

    pub(super) fn clear_last_error(last_error: &Arc<RwLock<Option<String>>>) {
        if let Ok(mut error) = last_error.write() {
            *error = None;
        }
    }
}
