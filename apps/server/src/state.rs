use crate::reporter::{MediaMetadataData, PlaybackStateData, ReporterProtocol, WindowInfoData};
use crate::storage::{AccessSettings, PersistedRuntimeState, Storage, UpstreamSettings};
use serde::Serialize;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::watch;
use tracing::{info, warn};

const DEFAULT_ACTIVITY_CAP: usize = 120;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    DesktopReporter,
    Mobile,
}

impl ClientKind {
    fn label(self) -> &'static str {
        match self {
            ClientKind::DesktopReporter => "Desktop Reporter",
            ClientKind::Mobile => "Mobile",
        }
    }

    fn panel_key(self) -> &'static str {
        match self {
            ClientKind::DesktopReporter => "desktop",
            ClientKind::Mobile => "mobile",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientEntry {
    pub id: u64,
    pub kind: ClientKind,
    pub connected_at: u64,
    pub client_info: Option<String>,
    pub device_id: Option<String>,
    pub session_id: u64,
    pub last_window: Option<String>,
    pub last_media: Option<String>,
    pub messages: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityEntry {
    pub ts: u64,
    pub kind: &'static str,
    pub client: Option<ClientKind>,
    pub client_id: Option<u64>,
    pub summary: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigSnapshot {
    pub upstream_enabled: bool,
    pub upstream_protocol: String,
    pub media_reporting_enabled: bool,
    pub s3_enabled: bool,
    pub native_configured: bool,
    pub mix_space_configured: bool,
    pub desktop_accepts_clients: bool,
    pub mobile_accepts_clients: bool,
}

#[derive(Debug, Serialize)]
pub struct StatsSnapshot {
    pub total_messages: u64,
    pub window_info_count: u64,
    pub media_playback_count: u64,
    pub artwork_uploads: u64,
    pub upstream_errors: u64,
    pub native_upstream_connections: u64,
    pub last_activity_at: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct DashboardSnapshot {
    pub started_at: u64,
    pub bind_addr: String,
    pub uptime_seconds: u64,
    pub config: ConfigSnapshot,
    pub stats: StatsSnapshot,
    pub clients: Vec<ClientEntry>,
    pub activity: Vec<ActivityEntry>,
    pub upstream: UpstreamSettings,
    pub access: AccessSettings,
}

// --- Panel-facing view types (match apps/panel/lib/status-data.ts) ---

#[derive(Debug, Clone, Default, Serialize)]
pub struct ActivityView {
    pub process_name: Option<String>,
    pub title: Option<String>,
    pub icon_url: Option<String>,
    pub app_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MediaView {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<f64>,
    pub elapsed_time: Option<f64>,
    pub playing: Option<bool>,
    pub artwork_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DeviceView {
    pub battery_level: Option<i32>,
    pub battery_charging: Option<bool>,
    pub network_wifi: Option<bool>,
    pub network_cellular: Option<bool>,
    pub network_vpn: Option<bool>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnlineStatusView {
    pub is_online: bool,
    pub client_kind: &'static str,
    pub device_info: Option<String>,
    pub last_activity_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsView {
    pub total_messages: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientSnapshot {
    pub status: OnlineStatusView,
    pub activity: Option<ActivityView>,
    pub media: Option<MediaView>,
    pub device: Option<DeviceView>,
    pub stats: StatsView,
}

#[derive(Debug, Clone)]
pub struct CachedAsset {
    pub mime_type: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct DesktopRuntime {
    activity: Option<ActivityView>,
    media: Option<MediaView>,
    last_activity_at: Option<u64>,
}

#[derive(Debug, Clone, Default)]
struct MobileRuntime {
    activity: Option<ActivityView>,
    media: Option<MediaView>,
    device: Option<DeviceView>,
    last_activity_at: Option<u64>,
}

#[derive(Debug, Clone, Default)]
struct PublicState {
    current_window: Option<String>,
    current_media: Option<String>,
}

#[derive(Debug, Default)]
struct AssetCache {
    dir: PathBuf,
}

impl AssetCache {
    fn path_for(&self, id: &str, mime_type: &str) -> PathBuf {
        self.category_dir(id).join(format!(
            "{}.{}",
            asset_file_name(id),
            asset_extension(mime_type)
        ))
    }

    fn find(&self, id: &str) -> Option<CachedAsset> {
        [
            ("image/png", "png"),
            ("image/jpeg", "jpg"),
            ("image/webp", "webp"),
            ("application/octet-stream", "bin"),
        ]
        .into_iter()
        .map(|(mime_type, extension)| {
            (
                mime_type,
                self.category_dir(id)
                    .join(format!("{}.{}", asset_file_name(id), extension)),
            )
        })
        .find(|(_, path)| path.is_file())
        .map(|(mime_type, path)| CachedAsset {
            mime_type: mime_type.to_string(),
            path,
        })
    }

    fn category_dir(&self, id: &str) -> PathBuf {
        self.dir.join(asset_category(id))
    }

    fn ensure_dirs(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.dir.join("app-icon"))?;
        fs::create_dir_all(self.dir.join("album-icon"))?;
        Ok(())
    }
}

pub struct DashboardState {
    started_at: Instant,
    started_at_unix: u64,
    bind_addr: String,
    storage: Storage,
    upstream_settings: Mutex<UpstreamSettings>,
    access_settings: Mutex<AccessSettings>,

    total_messages: AtomicU64,
    window_info_count: AtomicU64,
    media_playback_count: AtomicU64,
    artwork_uploads: AtomicU64,
    upstream_errors: AtomicU64,
    native_upstream_connections: AtomicU64,
    desktop_messages: AtomicU64,
    mobile_messages: AtomicU64,
    last_activity_at: AtomicU64,
    public_state: Mutex<PublicState>,

    desktop_runtime: Mutex<DesktopRuntime>,
    mobile_runtime: Mutex<MobileRuntime>,

    next_client_id: AtomicU64,
    next_session_id: AtomicU64,
    clients: Mutex<Vec<ClientEntry>>,
    activity: Mutex<VecDeque<ActivityEntry>>,
    assets: Mutex<AssetCache>,
    share_updates: watch::Sender<u64>,
}

impl DashboardState {
    pub fn new(bind_addr: String, storage: Storage) -> Self {
        let upstream_settings = storage.load_upstream_settings();
        let access_settings = storage.load_access_settings();
        let runtime_state = storage.load_runtime_state();
        let cap = access_settings.activity_log_limit.max(1) as usize;
        let activity = storage
            .load_recent_activity(cap)
            .into_iter()
            .collect::<VecDeque<_>>();
        let (share_updates, _) = watch::channel(0);
        let asset_cache = AssetCache {
            dir: storage.asset_dir(),
        };
        if let Err(error) = asset_cache.ensure_dirs() {
            warn!(
                "failed to create asset cache dir: path={}, error={error}",
                asset_cache.dir.display()
            );
        } else {
            info!("asset cache dir ready: path={}", asset_cache.dir.display());
        }
        Self {
            started_at: Instant::now(),
            started_at_unix: now_unix(),
            bind_addr,
            storage,
            upstream_settings: Mutex::new(upstream_settings),
            access_settings: Mutex::new(access_settings),
            total_messages: AtomicU64::new(runtime_state.total_messages),
            window_info_count: AtomicU64::new(runtime_state.window_info_count),
            media_playback_count: AtomicU64::new(runtime_state.media_playback_count),
            artwork_uploads: AtomicU64::new(runtime_state.artwork_uploads),
            upstream_errors: AtomicU64::new(runtime_state.upstream_errors),
            native_upstream_connections: AtomicU64::new(0),
            desktop_messages: AtomicU64::new(runtime_state.desktop_messages),
            mobile_messages: AtomicU64::new(runtime_state.mobile_messages),
            last_activity_at: AtomicU64::new(runtime_state.last_activity_at.unwrap_or_default()),
            public_state: Mutex::new(PublicState {
                current_window: runtime_state.current_window,
                current_media: runtime_state.current_media,
            }),
            desktop_runtime: Mutex::new(DesktopRuntime::default()),
            mobile_runtime: Mutex::new(MobileRuntime::default()),
            next_client_id: AtomicU64::new(1),
            next_session_id: AtomicU64::new(1),
            clients: Mutex::new(Vec::new()),
            activity: Mutex::new(activity),
            assets: Mutex::new(asset_cache),
            share_updates,
        }
    }

    pub fn snapshot(&self) -> DashboardSnapshot {
        let clients = self
            .clients
            .lock()
            .map(|clients| clients.clone())
            .unwrap_or_default();
        let activity = self
            .activity
            .lock()
            .map(|activity| activity.iter().cloned().collect())
            .unwrap_or_default();
        let upstream = self.upstream_settings();
        let access = self.access_settings();

        DashboardSnapshot {
            started_at: self.started_at_unix,
            bind_addr: self.bind_addr.clone(),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            config: build_config_snapshot(&upstream, &access),
            stats: StatsSnapshot {
                total_messages: self.total_messages.load(Ordering::Relaxed),
                window_info_count: self.window_info_count.load(Ordering::Relaxed),
                media_playback_count: self.media_playback_count.load(Ordering::Relaxed),
                artwork_uploads: self.artwork_uploads.load(Ordering::Relaxed),
                upstream_errors: self.upstream_errors.load(Ordering::Relaxed),
                native_upstream_connections: self
                    .native_upstream_connections
                    .load(Ordering::Relaxed),
                last_activity_at: {
                    let value = self.last_activity_at.load(Ordering::Relaxed);
                    if value == 0 { None } else { Some(value) }
                },
            },
            clients,
            activity,
            upstream,
            access,
        }
    }

    fn client_snapshots(&self) -> (ClientSnapshot, ClientSnapshot) {
        let (desktop_online, mobile_online) = self
            .clients
            .lock()
            .map(|clients| {
                (
                    clients
                        .iter()
                        .any(|c| c.kind == ClientKind::DesktopReporter),
                    clients.iter().any(|c| c.kind == ClientKind::Mobile),
                )
            })
            .unwrap_or((false, false));

        let (desktop_info, mobile_info) = self
            .clients
            .lock()
            .map(|clients| {
                let desktop = clients
                    .iter()
                    .find(|c| c.kind == ClientKind::DesktopReporter)
                    .and_then(|c| c.client_info.clone());
                let mobile = clients
                    .iter()
                    .find(|c| c.kind == ClientKind::Mobile)
                    .and_then(|c| c.client_info.clone());
                (desktop, mobile)
            })
            .unwrap_or((None, None));

        let (desktop_runtime, mobile_runtime) = (
            self.desktop_runtime
                .lock()
                .map(|r| r.clone())
                .unwrap_or_default(),
            self.mobile_runtime
                .lock()
                .map(|r| r.clone())
                .unwrap_or_default(),
        );

        let desktop_messages = self.desktop_messages.load(Ordering::Relaxed);
        let mobile_messages = self.mobile_messages.load(Ordering::Relaxed);
        let desktop_last_activity = desktop_runtime.last_activity_at.map(unix_to_iso);
        let mobile_last_activity = mobile_runtime.last_activity_at.map(unix_to_iso);

        (
            ClientSnapshot {
                status: OnlineStatusView {
                    is_online: desktop_online,
                    client_kind: ClientKind::DesktopReporter.panel_key(),
                    device_info: desktop_info,
                    last_activity_at: desktop_last_activity,
                },
                activity: desktop_runtime.activity,
                media: desktop_runtime.media,
                device: None,
                stats: StatsView {
                    total_messages: desktop_messages,
                },
            },
            ClientSnapshot {
                status: OnlineStatusView {
                    is_online: mobile_online,
                    client_kind: ClientKind::Mobile.panel_key(),
                    device_info: mobile_info,
                    last_activity_at: mobile_last_activity,
                },
                activity: mobile_runtime.activity,
                media: mobile_runtime.media,
                device: mobile_runtime.device,
                stats: StatsView {
                    total_messages: mobile_messages,
                },
            },
        )
    }

    pub fn desktop_share_snapshot(&self) -> ClientSnapshot {
        self.client_snapshots().0
    }

    pub fn mobile_share_snapshot(&self) -> ClientSnapshot {
        self.client_snapshots().1
    }

    pub fn subscribe_share_updates(&self) -> watch::Receiver<u64> {
        self.share_updates.subscribe()
    }

    pub fn cache_asset(&self, id: String, mime_type: String, data: Vec<u8>) {
        if id.trim().is_empty() {
            return;
        }

        if let Ok(assets) = self.assets.lock() {
            let _ = assets.ensure_dirs();
            let path = assets.path_for(&id, &mime_type);
            if fs::write(&path, data).is_err() {
                return;
            }

            info!("cached mobile asset: id={id}, path={}", path.display());
        }
    }

    pub fn cached_asset(&self, id: &str) -> Option<CachedAsset> {
        self.assets.lock().ok().and_then(|assets| assets.find(id))
    }

    fn persist_runtime_state(&self) {
        let public_state = self
            .public_state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_default();
        let last_activity_at = self.last_activity_at.load(Ordering::Relaxed);
        let _ = self.storage.save_runtime_state(&PersistedRuntimeState {
            total_messages: self.total_messages.load(Ordering::Relaxed),
            window_info_count: self.window_info_count.load(Ordering::Relaxed),
            media_playback_count: self.media_playback_count.load(Ordering::Relaxed),
            artwork_uploads: self.artwork_uploads.load(Ordering::Relaxed),
            upstream_errors: self.upstream_errors.load(Ordering::Relaxed),
            desktop_messages: self.desktop_messages.load(Ordering::Relaxed),
            mobile_messages: self.mobile_messages.load(Ordering::Relaxed),
            last_activity_at: if last_activity_at == 0 {
                None
            } else {
                Some(last_activity_at)
            },
            current_window: public_state.current_window,
            current_media: public_state.current_media,
        });
        self.notify_share_changed();
    }

    fn update_public_state(&self, current_window: Option<String>, current_media: Option<String>) {
        if let Ok(mut public_state) = self.public_state.lock() {
            if current_window.is_some() {
                public_state.current_window = current_window;
            }
            if current_media.is_some() {
                public_state.current_media = current_media;
            }
        }
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn upstream_settings(&self) -> UpstreamSettings {
        self.upstream_settings
            .lock()
            .map(|settings| settings.clone())
            .unwrap_or_default()
    }

    pub fn save_upstream_settings(&self, settings: UpstreamSettings) -> Result<(), String> {
        self.storage.save_upstream_settings(&settings)?;
        if let Ok(mut current) = self.upstream_settings.lock() {
            *current = settings;
        }
        let ts = now_unix();
        self.push_activity(ActivityEntry {
            ts,
            kind: "config_update",
            client: None,
            client_id: None,
            summary: "上游配置已更新".to_string(),
            detail: None,
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
        Ok(())
    }

    pub fn access_settings(&self) -> AccessSettings {
        self.access_settings
            .lock()
            .map(|settings| settings.clone())
            .unwrap_or_default()
    }

    pub fn save_access_settings(&self, settings: AccessSettings) -> Result<(), String> {
        let cap = settings.activity_log_limit.max(1) as usize;
        self.storage.save_access_settings(&settings)?;
        if let Ok(mut current) = self.access_settings.lock() {
            *current = settings;
        }
        // Trim the in-memory activity buffer if the new limit is smaller.
        if let Ok(mut activity) = self.activity.lock() {
            while activity.len() > cap {
                activity.pop_front();
            }
        }
        let ts = now_unix();
        self.push_activity(ActivityEntry {
            ts,
            kind: "config_update",
            client: None,
            client_id: None,
            summary: "接入控制配置已更新".to_string(),
            detail: None,
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
        Ok(())
    }

    pub fn add_client(
        &self,
        kind: ClientKind,
        client_info: Option<String>,
        device_id: Option<String>,
    ) -> Option<(u64, u64)> {
        let connected_at = now_unix();

        // 同一设备重连：复用原记录、复用客户端 ID，仅分配新会话 ID；
        // 旧 WebSocket 句柄即便尚活，其 close 时因 session_id 不匹配不会误删新会话。
        if let Some(ref did) = device_id {
            if let Ok(mut clients) = self.clients.lock() {
                if let Some(existing) = clients
                    .iter_mut()
                    .find(|c| c.device_id.as_deref() == Some(did.as_str()))
                {
                    let reused_id = existing.id;
                    let session_id = self.next_session_id.fetch_add(1, Ordering::Relaxed);
                    existing.session_id = session_id;
                    existing.connected_at = connected_at;
                    existing.kind = kind;
                    existing.client_info = client_info.clone();
                    existing.last_window = None;
                    existing.last_media = None;
                    existing.messages = 0;
                    self.push_activity(ActivityEntry {
                        ts: connected_at,
                        kind: "client_reconnect",
                        client: Some(kind),
                        client_id: Some(reused_id),
                        summary: client_info
                            .clone()
                            .unwrap_or_else(|| kind.label().to_string()),
                        detail: Some(format!("deviceId={did}")),
                    });
                    self.touch_activity(connected_at);
                    self.persist_runtime_state();
                    return Some((reused_id, session_id));
                }
            }
        }

        if self.has_client_kind(kind) {
            let ts = now_unix();
            self.push_activity(ActivityEntry {
                ts,
                kind: "client_rejected",
                client: Some(kind),
                client_id: None,
                summary: format!("{} 已在线，拒绝重复连接", kind.label()),
                detail: None,
            });
            self.touch_activity(ts);
            self.persist_runtime_state();
            return None;
        }

        let id = self.next_client_id.fetch_add(1, Ordering::Relaxed);
        let session_id = self.next_session_id.fetch_add(1, Ordering::Relaxed);
        let entry = ClientEntry {
            id,
            kind,
            connected_at,
            client_info: client_info.clone(),
            device_id,
            session_id,
            last_window: None,
            last_media: None,
            messages: 0,
        };
        if let Ok(mut clients) = self.clients.lock() {
            clients.push(entry);
        }
        self.push_activity(ActivityEntry {
            ts: connected_at,
            kind: "client_connect",
            client: Some(kind),
            client_id: Some(id),
            summary: client_info.unwrap_or_else(|| kind.label().to_string()),
            detail: None,
        });
        self.touch_activity(connected_at);
        self.persist_runtime_state();
        Some((id, session_id))
    }

    pub fn remove_client(&self, id: u64, session_id: u64, kind: ClientKind) {
        let removed = if let Ok(mut clients) = self.clients.lock() {
            let before = clients.len();
            clients.retain(|client| !(client.id == id && client.session_id == session_id));
            clients.len() != before
        } else {
            false
        };
        if removed {
            let ts = now_unix();
            self.push_activity(ActivityEntry {
                ts,
                kind: "client_disconnect",
                client: Some(kind),
                client_id: Some(id),
                summary: kind.label().to_string(),
                detail: None,
            });
            self.touch_activity(ts);
            self.persist_runtime_state();
        }
    }

    pub fn record_window(&self, client_id: u64, kind: ClientKind, data: &WindowInfoData) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.window_info_count.fetch_add(1, Ordering::Relaxed);
        self.bump_kind_messages(kind);
        let ts = now_unix();
        if let Ok(mut clients) = self.clients.lock()
            && let Some(client) = clients.iter_mut().find(|client| client.id == client_id)
        {
            client.messages = client.messages.saturating_add(1);
            client.last_window = Some(data.title.clone());
        }
        let title = data.title.clone();
        let process_name = data.process_name.clone();
        self.update_public_state(Some(title.clone()), None);

        let view = ActivityView {
            process_name: Some(process_name.clone()),
            title: Some(title.clone()),
            icon_url: icon_url_from_data(data),
            app_id: data.app_id.clone(),
        };
        match kind {
            ClientKind::DesktopReporter => {
                if let Ok(mut runtime) = self.desktop_runtime.lock() {
                    runtime.activity = Some(view.clone());
                    runtime.last_activity_at = Some(ts);
                }
            }
            ClientKind::Mobile => {
                if let Ok(mut runtime) = self.mobile_runtime.lock() {
                    runtime.activity = Some(view.clone());
                    runtime.last_activity_at = Some(ts);
                }
            }
        }

        let summary = if process_name.trim().is_empty() {
            title.clone()
        } else {
            format!("{process_name} · {title}")
        };
        self.push_activity(ActivityEntry {
            ts,
            kind: "window_info",
            client: Some(kind),
            client_id: Some(client_id),
            summary,
            detail: Some(title),
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
    }

    pub fn record_media(
        &self,
        client_id: u64,
        kind: ClientKind,
        metadata: &MediaMetadataData,
        playback: &PlaybackStateData,
    ) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.media_playback_count.fetch_add(1, Ordering::Relaxed);
        self.bump_kind_messages(kind);
        let ts = now_unix();
        if let Ok(mut clients) = self.clients.lock()
            && let Some(client) = clients.iter_mut().find(|client| client.id == client_id)
        {
            client.messages = client.messages.saturating_add(1);
            client.last_media = metadata.title.clone();
        }
        self.update_public_state(None, metadata.title.clone());

        let view = MediaView {
            title: metadata.title.clone(),
            artist: metadata.artist.clone(),
            album: metadata.album.clone(),
            duration: if metadata.duration > 0.0 {
                Some(metadata.duration)
            } else {
                None
            },
            elapsed_time: Some(playback.elapsed_time),
            playing: Some(playback.playing),
            artwork_url: metadata.artwork_url.clone(),
        };
        match kind {
            ClientKind::DesktopReporter => {
                if let Ok(mut runtime) = self.desktop_runtime.lock() {
                    runtime.media = Some(view);
                    runtime.last_activity_at = Some(ts);
                }
            }
            ClientKind::Mobile => {
                if let Ok(mut runtime) = self.mobile_runtime.lock() {
                    runtime.media = Some(view);
                    runtime.last_activity_at = Some(ts);
                }
            }
        }

        let summary = match (metadata.title.as_deref(), metadata.artist.as_deref()) {
            (Some(title), Some(artist)) if !title.is_empty() && !artist.is_empty() => {
                format!("{artist} - {title}")
            }
            (Some(title), _) if !title.is_empty() => title.to_string(),
            (None, Some(artist)) => artist.to_string(),
            _ => "媒体播放".to_string(),
        };
        self.push_activity(ActivityEntry {
            ts,
            kind: "media_playback",
            client: Some(kind),
            client_id: Some(client_id),
            summary,
            detail: None,
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
    }

    pub fn record_mobile_device(&self, device: DeviceView) {
        if let Ok(mut runtime) = self.mobile_runtime.lock() {
            runtime.device = Some(device);
            runtime.last_activity_at = Some(now_unix());
        }
        self.notify_share_changed();
    }

    pub fn record_artwork_upload(&self, client_id: u64, kind: ClientKind, content_id: &str) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.artwork_uploads.fetch_add(1, Ordering::Relaxed);
        self.bump_kind_messages(kind);
        let ts = now_unix();
        if let Ok(mut clients) = self.clients.lock()
            && let Some(client) = clients.iter_mut().find(|client| client.id == client_id)
        {
            client.messages = client.messages.saturating_add(1);
        }
        self.push_activity(ActivityEntry {
            ts,
            kind: "artwork_upload",
            client: Some(kind),
            client_id: Some(client_id),
            summary: content_id.to_string(),
            detail: None,
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
    }

    pub fn record_upstream_error(&self) {
        self.upstream_errors.fetch_add(1, Ordering::Relaxed);
        let ts = now_unix();
        self.push_activity(ActivityEntry {
            ts,
            kind: "upstream_error",
            client: None,
            client_id: None,
            summary: "上游转发失败".to_string(),
            detail: None,
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
    }

    pub fn inc_native_upstream(&self) {
        self.native_upstream_connections
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_native_upstream(&self) {
        self.native_upstream_connections
            .fetch_sub(1, Ordering::Relaxed);
    }

    pub fn clear_activity_log(&self) -> Result<(), String> {
        self.storage.clear_activity()?;
        if let Ok(mut activity) = self.activity.lock() {
            activity.clear();
        }
        let ts = now_unix();
        self.push_activity(ActivityEntry {
            ts,
            kind: "config_update",
            client: None,
            client_id: None,
            summary: "活动日志已清空".to_string(),
            detail: None,
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
        Ok(())
    }

    pub fn reset_stats(&self) -> Result<(), String> {
        self.total_messages.store(0, Ordering::Relaxed);
        self.window_info_count.store(0, Ordering::Relaxed);
        self.media_playback_count.store(0, Ordering::Relaxed);
        self.artwork_uploads.store(0, Ordering::Relaxed);
        self.upstream_errors.store(0, Ordering::Relaxed);
        self.desktop_messages.store(0, Ordering::Relaxed);
        self.mobile_messages.store(0, Ordering::Relaxed);
        self.last_activity_at.store(0, Ordering::Relaxed);
        if let Ok(mut public_state) = self.public_state.lock() {
            public_state.current_window = None;
            public_state.current_media = None;
        }
        if let Ok(mut runtime) = self.desktop_runtime.lock() {
            runtime.activity = None;
            runtime.media = None;
            runtime.last_activity_at = None;
        }
        if let Ok(mut runtime) = self.mobile_runtime.lock() {
            runtime.activity = None;
            runtime.media = None;
            runtime.device = None;
            runtime.last_activity_at = None;
        }
        self.persist_runtime_state();
        let ts = now_unix();
        self.push_activity(ActivityEntry {
            ts,
            kind: "config_update",
            client: None,
            client_id: None,
            summary: "运行时统计已重置".to_string(),
            detail: None,
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
        Ok(())
    }

    fn touch_activity(&self, ts: u64) {
        self.last_activity_at.store(ts, Ordering::Relaxed);
    }

    fn bump_kind_messages(&self, kind: ClientKind) {
        match kind {
            ClientKind::DesktopReporter => {
                self.desktop_messages.fetch_add(1, Ordering::Relaxed);
            }
            ClientKind::Mobile => {
                self.mobile_messages.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn push_activity(&self, entry: ActivityEntry) {
        self.storage.record_activity(&entry);
        if let Ok(mut activity) = self.activity.lock() {
            let cap = self.activity_cap();
            while activity.len() >= cap {
                activity.pop_front();
            }
            activity.push_back(entry);
        }
    }

    fn notify_share_changed(&self) {
        let next_version = self.share_updates.borrow().wrapping_add(1);
        let _ = self.share_updates.send(next_version);
    }

    fn activity_cap(&self) -> usize {
        self.access_settings
            .lock()
            .map(|settings| settings.activity_log_limit.max(1) as usize)
            .unwrap_or(DEFAULT_ACTIVITY_CAP)
    }

    fn has_client_kind(&self, kind: ClientKind) -> bool {
        self.clients
            .lock()
            .map(|clients| clients.iter().any(|client| client.kind == kind))
            .unwrap_or(false)
    }
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn unix_to_iso(ts: u64) -> String {
    let secs = ts as i64;
    let (y, m, d, hh, mm, ss) = civil_from_unix(secs);
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

fn asset_file_name(id: &str) -> String {
    id.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn asset_extension(mime_type: &str) -> &'static str {
    match mime_type {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => "bin",
    }
}

fn asset_category(id: &str) -> &'static str {
    if id.starts_with("media-artwork:") {
        "album-icon"
    } else {
        "app-icon"
    }
}

/// Convert a unix timestamp (seconds) to UTC civil date/time components.
fn civil_from_unix(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let hh = (rem / 3600) as u32;
    let mm = ((rem % 3600) / 60) as u32;
    let ss = (rem % 60) as u32;

    // Howard Hinnant's civil_from_days algorithm.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (y + if m <= 2 { 1 } else { 0 }, m, d, hh, mm, ss)
}

/// Build an `icon_url` for a window info payload, preferring an explicit URL and
/// falling back to a base64 data URI when only `icon_base64` is provided.
fn icon_url_from_data(data: &WindowInfoData) -> Option<String> {
    if let Some(url) = data.icon_url.as_ref()
        && !url.trim().is_empty()
    {
        return Some(url.clone());
    }
    let b64 = data.icon_base64.as_ref()?;
    if b64.trim().is_empty() {
        return None;
    }
    Some(format!("data:image/png;base64,{b64}"))
}

pub type SharedDashboardState = Arc<DashboardState>;

fn build_config_snapshot(settings: &UpstreamSettings, access: &AccessSettings) -> ConfigSnapshot {
    let native_configured =
        !settings.native_ws_url.trim().is_empty() && !settings.native_token.trim().is_empty();
    let mix_space_configured = !settings.mix_space_endpoint.trim().is_empty()
        && !settings.mix_space_token.trim().is_empty();
    let upstream_enabled = match settings.protocol {
        ReporterProtocol::Native => native_configured,
        ReporterProtocol::MixSpace => mix_space_configured,
    };

    ConfigSnapshot {
        upstream_enabled,
        upstream_protocol: match settings.protocol {
            ReporterProtocol::Native => "native".to_string(),
            ReporterProtocol::MixSpace => "mix_space".to_string(),
        },
        media_reporting_enabled: settings.enable_media_reporting,
        s3_enabled: settings.s3_enabled,
        native_configured,
        mix_space_configured,
        desktop_accepts_clients: access.accept_desktop,
        mobile_accepts_clients: access.accept_mobile,
    }
}
