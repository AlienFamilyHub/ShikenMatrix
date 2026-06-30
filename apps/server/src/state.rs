use crate::reporter::ReporterProtocol;
use crate::storage::{PersistedRuntimeState, Storage, UpstreamSettings};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const ACTIVITY_CAP: usize = 120;

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
}

#[derive(Debug, Serialize)]
pub struct PublicSnapshot {
    pub current_window: Option<String>,
    pub current_media: Option<String>,
    pub last_activity_at: Option<u64>,
}

#[derive(Debug, Clone, Default)]
struct PublicState {
    current_window: Option<String>,
    current_media: Option<String>,
}

pub struct DashboardState {
    started_at: Instant,
    started_at_unix: u64,
    bind_addr: String,
    storage: Storage,
    upstream_settings: Mutex<UpstreamSettings>,

    total_messages: AtomicU64,
    window_info_count: AtomicU64,
    media_playback_count: AtomicU64,
    artwork_uploads: AtomicU64,
    upstream_errors: AtomicU64,
    native_upstream_connections: AtomicU64,
    last_activity_at: AtomicU64,
    public_state: Mutex<PublicState>,

    next_client_id: AtomicU64,
    next_session_id: AtomicU64,
    clients: Mutex<Vec<ClientEntry>>,
    activity: Mutex<VecDeque<ActivityEntry>>,
}

impl DashboardState {
    pub fn new(bind_addr: String, storage: Storage) -> Self {
        let upstream_settings = storage.load_upstream_settings();
        let runtime_state = storage.load_runtime_state();
        let activity = storage
            .load_recent_activity(ACTIVITY_CAP)
            .into_iter()
            .collect::<VecDeque<_>>();
        Self {
            started_at: Instant::now(),
            started_at_unix: now_unix(),
            bind_addr,
            storage,
            upstream_settings: Mutex::new(upstream_settings),
            total_messages: AtomicU64::new(runtime_state.total_messages),
            window_info_count: AtomicU64::new(runtime_state.window_info_count),
            media_playback_count: AtomicU64::new(runtime_state.media_playback_count),
            artwork_uploads: AtomicU64::new(runtime_state.artwork_uploads),
            upstream_errors: AtomicU64::new(runtime_state.upstream_errors),
            native_upstream_connections: AtomicU64::new(0),
            last_activity_at: AtomicU64::new(runtime_state.last_activity_at.unwrap_or_default()),
            public_state: Mutex::new(PublicState {
                current_window: runtime_state.current_window,
                current_media: runtime_state.current_media,
            }),
            next_client_id: AtomicU64::new(1),
            next_session_id: AtomicU64::new(1),
            clients: Mutex::new(Vec::new()),
            activity: Mutex::new(activity),
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

        DashboardSnapshot {
            started_at: self.started_at_unix,
            bind_addr: self.bind_addr.clone(),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            config: build_config_snapshot(&upstream),
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
        }
    }

    pub fn public_snapshot(&self) -> PublicSnapshot {
        let public_state = self
            .public_state
            .lock()
            .map(|public_state| public_state.clone())
            .unwrap_or_default();
        let last_activity = self.last_activity_at.load(Ordering::Relaxed);

        PublicSnapshot {
            current_window: public_state.current_window,
            current_media: public_state.current_media,
            last_activity_at: if last_activity == 0 {
                None
            } else {
                Some(last_activity)
            },
        }
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
            last_activity_at: if last_activity_at == 0 {
                None
            } else {
                Some(last_activity_at)
            },
            current_window: public_state.current_window,
            current_media: public_state.current_media,
        });
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
                if let Some(existing) = clients.iter_mut().find(|c| c.device_id.as_deref() == Some(did.as_str())) {
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

    pub fn record_window_info(
        &self,
        client_id: u64,
        kind: ClientKind,
        title: &str,
        process_name: &str,
    ) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.window_info_count.fetch_add(1, Ordering::Relaxed);
        let ts = now_unix();
        if let Ok(mut clients) = self.clients.lock()
            && let Some(client) = clients.iter_mut().find(|client| client.id == client_id)
        {
            client.messages = client.messages.saturating_add(1);
            client.last_window = Some(title.to_string());
        }
        self.update_public_state(Some(title.to_string()), None);
        let summary = if process_name.trim().is_empty() {
            title.to_string()
        } else {
            format!("{process_name} · {title}")
        };
        self.push_activity(ActivityEntry {
            ts,
            kind: "window_info",
            client: Some(kind),
            client_id: Some(client_id),
            summary,
            detail: Some(title.to_string()),
        });
        self.touch_activity(ts);
        self.persist_runtime_state();
    }

    pub fn record_media_playback(
        &self,
        client_id: u64,
        kind: ClientKind,
        title: Option<&str>,
        artist: Option<&str>,
    ) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.media_playback_count.fetch_add(1, Ordering::Relaxed);
        let ts = now_unix();
        if let Ok(mut clients) = self.clients.lock()
            && let Some(client) = clients.iter_mut().find(|client| client.id == client_id)
        {
            client.messages = client.messages.saturating_add(1);
            client.last_media = Some(title.unwrap_or("").to_string());
        }
        self.update_public_state(None, title.map(ToString::to_string));
        let summary = match (title, artist) {
            (Some(title), Some(artist)) if !title.is_empty() && !artist.is_empty() => {
                format!("{artist} - {title}")
            }
            (Some(title), _) => title.to_string(),
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

    pub fn record_artwork_upload(&self, client_id: u64, kind: ClientKind, content_id: &str) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.artwork_uploads.fetch_add(1, Ordering::Relaxed);
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

    fn touch_activity(&self, ts: u64) {
        self.last_activity_at.store(ts, Ordering::Relaxed);
    }

    fn push_activity(&self, entry: ActivityEntry) {
        self.storage.record_activity(&entry);
        if let Ok(mut activity) = self.activity.lock() {
            if activity.len() >= ACTIVITY_CAP {
                activity.pop_front();
            }
            activity.push_back(entry);
        }
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

pub type SharedDashboardState = Arc<DashboardState>;

fn build_config_snapshot(settings: &UpstreamSettings) -> ConfigSnapshot {
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
        desktop_accepts_clients: true,
        mobile_accepts_clients: true,
    }
}
