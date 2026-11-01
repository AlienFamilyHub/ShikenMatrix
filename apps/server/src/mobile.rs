use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::reporter::{
    MediaMetadataData, MediaPlaybackMessage, PlaybackStateData, ReporterMessage, ReporterProtocol,
    UploadArtworkMetaMessage, WindowInfoData, WindowInfoMessage, run_mix_space_reporter,
    run_native_reporter,
};
use crate::state::{ClientKind, DeviceView, SharedDashboardState};

const MOBILE_IDLE_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(90);

pub async fn mobile_ws(
    State(state): State<SharedDashboardState>,
    axum::extract::Query(query): axum::extract::Query<MobileQuery>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    let key = query.key.unwrap_or_default();
    if !state.storage().verify_client_key(&key) {
        return (axum::http::StatusCode::UNAUTHORIZED, "invalid client key").into_response();
    }
    if !state.access_settings().accept_mobile {
        return (
            axum::http::StatusCode::FORBIDDEN,
            "mobile clients not accepted",
        )
            .into_response();
    }
    let session = MobileSession {
        client: query.client,
        device_id: query.device_id,
    };
    upgrade.on_upgrade(move |socket| handle_mobile_socket(socket, state, session))
}

#[derive(serde::Deserialize)]
pub(crate) struct MobileQuery {
    key: Option<String>,
    #[serde(default)]
    client: Option<String>,
    #[serde(default, rename = "deviceId")]
    device_id: Option<String>,
}

async fn handle_mobile_socket(
    mut socket: WebSocket,
    state: SharedDashboardState,
    session: MobileSession,
) {
    info!(
        "mobile session established: client={:?}, device_id={:?}",
        session.client, session.device_id
    );
    let Some((client_id, session_token)) = state.add_client(
        ClientKind::Mobile,
        session.client.clone(),
        session.device_id.clone(),
    ) else {
        let _ = socket
            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                code: 1008,
                reason: "mobile already connected".into(),
            })))
            .await;
        return;
    };
    let reporter_tx = start_mobile_reporter(&state);
    let mut pending_artwork_meta: Option<UploadArtworkMetaMessage> = None;

    // 心跳：屏幕休眠后 NAT/系统可能静默丢连接，用 ping + idle timeout 尽快释放半开连接。
    let (mut socket_tx, mut socket_rx) = socket.split();
    let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_secs(20));
    heartbeat.reset();
    let mut last_seen = std::time::Instant::now();

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if socket_tx.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
                if last_seen.elapsed() > MOBILE_IDLE_TIMEOUT {
                    warn!("mobile client idle timeout, closing");
                    let _ = socket_tx
                        .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                            code: 1001,
                            reason: "idle timeout".into(),
                        })))
                        .await;
                    break;
                }
            }
            Some(message) = socket_rx.next() => {
                match message {
                    Ok(Message::Text(text)) => {
                        last_seen = std::time::Instant::now();
                        if let Ok(query) = serde_json::from_str::<MobileAssetCacheQuery>(text.as_str()) {
                            if query.message_type == "asset_cache_query" {
                                let cached = state.cached_asset(&query.content_item_identifier).is_some();
                                if !cached {
                                    pending_artwork_meta = Some(UploadArtworkMetaMessage {
                                        content_item_identifier: query.content_item_identifier.clone(),
                                        mime_type: query.mime_type.clone(),
                                    });
                                }
                                let response = MobileAssetCacheStatus {
                                    message_type: "asset_cache_status",
                                    content_item_identifier: query.content_item_identifier,
                                    cached,
                                };
                                if let Ok(payload) = serde_json::to_string(&response) {
                                    let _ = socket_tx.send(Message::Text(payload.into())).await;
                                }
                                continue;
                            }
                        }

                        match parse_mobile_payload(&session, text.as_str()) {
                            Ok(payload) => {
                                info!("mobile payload received: {}", payload.message_type);
                                forward_mobile_payload(payload, reporter_tx.as_ref(), &state, client_id, session_token);
                            }
                            Err(error) => warn!("mobile payload parse failed: {error}"),
                        }
                        let _ = socket_tx
                            .send(Message::Text(r#"{"type":"mobile_payload_ack"}"#.into()))
                            .await;
                    }
                    Ok(Message::Binary(data)) => {
                        last_seen = std::time::Instant::now();
                        if let Some(meta) = pending_artwork_meta.take() {
                            state.cache_asset(
                                meta.content_item_identifier.clone(),
                                meta.mime_type.clone(),
                                data.to_vec(),
                            );
                            state.record_artwork_upload(
                                client_id,
                                ClientKind::Mobile,
                                &meta.content_item_identifier,
                            );
                            if let Some(reporter_tx) = reporter_tx.as_ref() {
                                let _ = reporter_tx.send(ReporterMessage::UploadArtwork {
                                    content_item_identifier: meta.content_item_identifier,
                                    artwork_data: data.to_vec(),
                                    mime_type: meta.mime_type,
                                });
                            }
                        }
                    }
                    Ok(Message::Pong(_)) | Ok(Message::Ping(_)) => {
                        last_seen = std::time::Instant::now();
                    }
                    Ok(Message::Close(_)) => break,
                    Err(error) => {
                        info!("mobile websocket closed: {error}");
                        break;
                    }
                }
            }
            else => break,
        }
    }

    state.remove_client(client_id, session_token, ClientKind::Mobile);
}

fn parse_mobile_payload(_session: &MobileSession, text: &str) -> Result<MobilePayload, String> {
    if let Ok(payload) = serde_json::from_str::<MobilePayload>(text) {
        return Ok(payload);
    }

    Err("payload is not android_snapshot JSON".to_string())
}

fn start_mobile_reporter(
    state: &SharedDashboardState,
) -> Option<mpsc::UnboundedSender<ReporterMessage>> {
    let config = state.upstream_settings().to_reporter_config()?;
    let (reporter_tx, reporter_rx) = mpsc::unbounded_channel();

    match config.protocol {
        ReporterProtocol::Native => {
            let (server_tx, _server_rx) = mpsc::unbounded_channel();
            tokio::spawn(run_native_reporter(
                config,
                reporter_rx,
                server_tx,
                state.clone(),
            ));
        }
        ReporterProtocol::MixSpace => {
            tokio::spawn(run_mix_space_reporter(config, reporter_rx, state.clone()));
        }
    }

    Some(reporter_tx)
}

fn forward_mobile_payload(
    payload: MobilePayload,
    reporter_tx: Option<&mpsc::UnboundedSender<ReporterMessage>>,
    state: &SharedDashboardState,
    client_id: u64,
    session_token: u64,
) {
    if payload.message_type != "android_snapshot" {
        return;
    }

    let foreground = payload.snapshot.foreground_app.unwrap_or_default();
    let foreground_icon_url = foreground
        .app_icon
        .as_ref()
        .and_then(|asset| asset_url_from_ref(state, asset));
    let process_name = foreground
        .package_name
        .clone()
        .unwrap_or_else(|| "android".to_string());
    let title = foreground
        .label
        .or_else(|| foreground.package_name.clone())
        .unwrap_or_else(|| "Android".to_string());

    let window_data = WindowInfoData {
        title: title.clone(),
        process_name: process_name.clone(),
        icon_base64: None,
        icon_url: foreground_icon_url,
        app_id: foreground.package_name,
        pid: 0,
    };

    let media_data = payload.snapshot.media.map(|media| {
        let artwork_url = media
            .artwork
            .as_ref()
            .and_then(|asset| asset_url_from_ref(state, asset))
            .or_else(|| {
                media
                    .app_icon
                    .as_ref()
                    .and_then(|asset| asset_url_from_ref(state, asset))
            });
        let content_item_identifier = media
            .artwork
            .as_ref()
            .and_then(|asset| asset.content_item_identifier.clone());
        let metadata = MediaMetadataData {
            bundle_identifier: media.package_name,
            title: media.title,
            artist: media.artist,
            album: media.album,
            duration: millis_to_seconds(media.duration),
            artwork_url,
            content_item_identifier,
        };
        let playback = PlaybackStateData {
            playing: media.state == Some(3),
            playback_rate: if media.state == Some(3) { 1.0 } else { 0.0 },
            elapsed_time: millis_to_seconds(media.position),
        };
        (metadata, playback)
    });

    let mut device = DeviceView::default();
    if let Some(battery) = payload.snapshot.battery {
        device.battery_level = Some(battery.level);
        device.battery_charging = Some(battery.charging);
    }
    if let Some(network) = payload.snapshot.network {
        device.network_wifi = Some(network.wifi);
        device.network_cellular = Some(network.cellular);
        device.network_vpn = Some(network.vpn);
    }
    if let Some(location) = payload.snapshot.coarse_location {
        device.latitude = location.latitude;
        device.longitude = location.longitude;
    }
    if !state.record_mobile_snapshot(
        client_id,
        session_token,
        payload.snapshot.screen_off,
        &window_data,
        media_data.as_ref().map(|(metadata, playback)| (metadata, playback)),
        device,
    ) {
        return;
    }
    if let Some(reporter_tx) = reporter_tx {
        let _ = reporter_tx.send(ReporterMessage::WindowInfo(WindowInfoMessage {
            data: window_data,
        }));
        if let Some((metadata, playback_state)) = media_data {
            let _ = reporter_tx.send(ReporterMessage::MediaPlayback(MediaPlaybackMessage {
                metadata,
                playback_state,
            }));
        }
    }
}

#[derive(Debug)]
struct MobileSession {
    client: Option<String>,
    device_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MobileAssetCacheQuery {
    #[serde(rename = "type")]
    message_type: String,
    content_item_identifier: String,
    mime_type: String,
}

#[derive(Debug, Serialize)]
struct MobileAssetCacheStatus {
    #[serde(rename = "type")]
    message_type: &'static str,
    content_item_identifier: String,
    cached: bool,
}

#[derive(Debug, Deserialize)]
struct MobilePayload {
    #[serde(rename = "type")]
    message_type: String,
    snapshot: AndroidSnapshot,
}

#[derive(Debug, Default, Deserialize)]
struct AndroidSnapshot {
    #[serde(default, rename = "screenOff")]
    screen_off: Option<bool>,
    #[serde(default, rename = "foregroundApp")]
    foreground_app: Option<ForegroundApp>,
    #[serde(default)]
    media: Option<MediaSnapshot>,
    #[serde(default)]
    battery: Option<BatterySnapshot>,
    #[serde(default)]
    network: Option<NetworkSnapshot>,
    #[serde(default, rename = "coarseLocation")]
    coarse_location: Option<CoarseLocationSnapshot>,
}

#[derive(Debug, Default, Deserialize)]
struct ForegroundApp {
    #[serde(default, rename = "packageName")]
    package_name: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default, rename = "appIcon")]
    app_icon: Option<MobileAssetRef>,
}

#[derive(Debug, Deserialize)]
struct MediaSnapshot {
    #[serde(default, rename = "packageName")]
    package_name: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    artist: Option<String>,
    #[serde(default)]
    album: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    position: Option<f64>,
    #[serde(default)]
    state: Option<i32>,
    #[serde(default)]
    artwork: Option<MobileAssetRef>,
    #[serde(default, rename = "appIcon")]
    app_icon: Option<MobileAssetRef>,
}

#[derive(Debug, Default, Deserialize)]
struct BatterySnapshot {
    #[serde(default)]
    level: i32,
    #[serde(default)]
    charging: bool,
}

#[derive(Debug, Default, Deserialize)]
struct NetworkSnapshot {
    #[serde(default)]
    wifi: bool,
    #[serde(default)]
    cellular: bool,
    #[serde(default)]
    vpn: bool,
}

#[derive(Debug, Default, Deserialize)]
struct CoarseLocationSnapshot {
    #[serde(default)]
    latitude: Option<f64>,
    #[serde(default)]
    longitude: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct MobileAssetRef {
    #[serde(default, rename = "contentItemIdentifier")]
    content_item_identifier: Option<String>,
}

fn millis_to_seconds(value: Option<f64>) -> f64 {
    value.unwrap_or_default() / 1000.0
}

fn asset_url_from_ref(state: &SharedDashboardState, asset: &MobileAssetRef) -> Option<String> {
    let id = asset.content_item_identifier.as_ref()?;
    state
        .cached_asset(id)
        .map(|_| format!("/api/assets?id={}", url_escape(id)))
}

fn url_escape(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::DashboardState;
    use crate::storage::Storage;
    use serde_json::{Value, json};
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory(PathBuf);

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).expect("remove test directory");
        }
    }

    struct TestServer {
        state: SharedDashboardState,
        _directory: TestDirectory,
    }

    impl TestServer {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "shikenmatrix-mobile-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).expect("create test directory");
            let directory = TestDirectory(path);
            let storage = Storage::open(directory.0.join("state.sqlite3")).expect("open storage");
            Self {
                state: Arc::new(DashboardState::new("127.0.0.1:0".to_string(), storage)),
                _directory: directory,
            }
        }

        fn connect(&self) -> (u64, u64) {
            self.state
                .add_client(
                    ClientKind::Mobile,
                    Some("Test Android".to_string()),
                    Some("test-phone".to_string()),
                )
                .expect("accept mobile connection")
        }

        fn deliver(&self, connection: (u64, u64), text: &str) {
            let session = MobileSession {
                client: Some("Test Android".to_string()),
                device_id: Some("test-phone".to_string()),
            };
            let payload = parse_mobile_payload(&session, text).expect("parse actual mobile JSON");
            forward_mobile_payload(payload, None, &self.state, connection.0, connection.1);
        }
    }

    async fn mobile_api(state: &SharedDashboardState) -> Value {
        let response = crate::panel::api_share_mobile(State(state.clone()))
            .await
            .into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read mobile API body");
        serde_json::from_slice(&body).expect("decode mobile API JSON")
    }

    #[tokio::test]
    async fn screen_off_remains_online_and_updates_public_device_and_media() {
        let server = TestServer::new();
        let connection = server.connect();
        server.deliver(
            connection,
            r#"{"type":"android_snapshot","snapshot":{
                "screenOff":false,
                "foregroundApp":{"packageName":"org.example.player","label":"Player"},
                "media":{"title":"Bright track","artist":"Artist","state":3,"position":10000},
                "battery":{"level":81,"charging":false},
                "network":{"wifi":true,"cellular":false,"vpn":false},
                "coarseLocation":{"latitude":31.2,"longitude":121.4}
            }}"#,
        );
        let bright = mobile_api(&server.state).await;
        assert_eq!(bright["status"]["is_online"], json!(true));
        assert_eq!(bright["status"].get("screen_off"), Some(&json!(false)));

        server.deliver(
            connection,
            r#"{"type":"android_snapshot","snapshot":{
                "screenOff":true,
                "foregroundApp":{"packageName":"org.example.player","label":"Player"},
                "media":{"title":"Background track","artist":"Artist","state":3,"position":20000},
                "battery":{"level":80,"charging":true},
                "network":{"wifi":false,"cellular":true,"vpn":true},
                "coarseLocation":{"latitude":31.3,"longitude":121.5}
            }}"#,
        );
        let off = mobile_api(&server.state).await;
        assert_eq!(off["status"]["is_online"], json!(true));
        assert_eq!(off["status"].get("screen_off"), Some(&json!(true)));
        assert_eq!(server.state.snapshot().clients.len(), 1);
        assert_eq!(off["media"]["title"], json!("Background track"));
        assert_eq!(off["media"]["elapsed_time"], json!(20.0));
        assert_eq!(off["media"]["playing"], json!(true));
        assert_eq!(off["device"]["battery_level"], json!(80));
        assert_eq!(off["device"]["battery_charging"], json!(true));
        assert_eq!(off["device"]["network_wifi"], json!(false));
        assert_eq!(off["device"]["network_cellular"], json!(true));
        assert_eq!(off["device"]["network_vpn"], json!(true));
        assert_eq!(off["device"]["latitude"], json!(31.3));
        assert_eq!(off["device"]["longitude"], json!(121.5));
        assert!(off["status"]["last_activity_at"].is_string());

        server.deliver(
            connection,
            r#"{"type":"android_snapshot","snapshot":{
                "screenOff":true,
                "media":{"title":"Next background track","state":3,"position":64000},
                "battery":{"level":79,"charging":false},
                "network":{"wifi":true,"cellular":true,"vpn":false},
                "coarseLocation":{"latitude":31.4,"longitude":121.6}
            }}"#,
        );
        let updated = mobile_api(&server.state).await;
        assert_eq!(updated["status"]["is_online"], json!(true));
        assert_eq!(updated["status"]["screen_off"], json!(true));
        assert_eq!(updated["media"]["title"], json!("Next background track"));
        assert_eq!(updated["media"]["elapsed_time"], json!(64.0));
        assert_eq!(updated["device"]["battery_level"], json!(79));
        assert_eq!(updated["device"]["battery_charging"], json!(false));
        assert_eq!(updated["device"]["network_wifi"], json!(true));
        assert_eq!(updated["device"]["network_vpn"], json!(false));
        assert_eq!(updated["device"]["latitude"], json!(31.4));
        assert_eq!(updated["device"]["longitude"], json!(121.6));
        assert!(
            updated["stats"]["total_messages"].as_u64().unwrap()
                > off["stats"]["total_messages"].as_u64().unwrap()
        );

        server.deliver(
            connection,
            r#"{"type":"android_snapshot","snapshot":{"screenOff":false}}"#,
        );
        let awake = mobile_api(&server.state).await;
        assert_eq!(awake["status"]["is_online"], json!(true));
        assert_eq!(awake["status"]["screen_off"], json!(false));
        assert_eq!(server.state.snapshot().clients.len(), 1);
    }

    #[tokio::test]
    async fn reconnect_clears_current_screen_and_rejects_old_session_updates() {
        let server = TestServer::new();
        let original = server.connect();
        server.deliver(
            original,
            r#"{"type":"android_snapshot","snapshot":{
                "screenOff":true,"battery":{"level":70,"charging":false}
            }}"#,
        );
        let replacement = server.connect();
        assert_eq!(replacement.0, original.0);
        assert_ne!(replacement.1, original.1);
        let reconnect = mobile_api(&server.state).await;
        assert_eq!(reconnect["status"]["is_online"], json!(true));
        assert_eq!(reconnect["status"].get("screen_off"), Some(&Value::Null));

        server.deliver(
            replacement,
            r#"{"type":"android_snapshot","snapshot":{
                "screenOff":false,
                "media":{"title":"Current session","state":3,"position":5000},
                "battery":{"level":69,"charging":true}
            }}"#,
        );
        let current = mobile_api(&server.state).await;
        server.deliver(
            original,
            r#"{"type":"android_snapshot","snapshot":{
                "screenOff":true,
                "media":{"title":"Stale session","state":2,"position":1000},
                "battery":{"level":1,"charging":false}
            }}"#,
        );
        server.state.remove_client(original.0, original.1, ClientKind::Mobile);
        assert_eq!(mobile_api(&server.state).await, current);

        server.deliver(
            replacement,
            r#"{"type":"android_snapshot","snapshot":{"screenOff":true}}"#,
        );
        let screen_off = mobile_api(&server.state).await;
        assert_eq!(screen_off["status"]["is_online"], json!(true));
        assert_eq!(screen_off["status"]["screen_off"], json!(true));
        server.state.remove_client(replacement.0, replacement.1, ClientKind::Mobile);
        let disconnected = mobile_api(&server.state).await;
        assert_eq!(disconnected["status"]["is_online"], json!(false));
        assert_eq!(disconnected["status"].get("screen_off"), Some(&Value::Null));
        server.deliver(
            replacement,
            r#"{"type":"android_snapshot","snapshot":{"screenOff":false}}"#,
        );
        assert_eq!(mobile_api(&server.state).await, disconnected);

        let next_connection = server.connect();
        let next = mobile_api(&server.state).await;
        assert_eq!(next["status"]["is_online"], json!(true));
        assert_eq!(next["status"].get("screen_off"), Some(&Value::Null));
        server.deliver(
            next_connection,
            r#"{"type":"android_snapshot","snapshot":{"screenOff":false}}"#,
        );
        assert_eq!(mobile_api(&server.state).await["status"]["screen_off"], json!(false));
    }

    #[tokio::test]
    async fn absent_screen_field_is_unknown_and_keeps_older_senders_online() {
        let server = TestServer::new();
        let initial = mobile_api(&server.state).await;
        assert_eq!(initial["status"].get("screen_off"), Some(&Value::Null));
        let connection = server.connect();
        server.deliver(
            connection,
            r#"{"type":"android_snapshot","snapshot":{"battery":{"level":42,"charging":false}}}"#,
        );
        let legacy = mobile_api(&server.state).await;
        assert_eq!(legacy["status"]["is_online"], json!(true));
        assert_eq!(legacy["status"].get("screen_off"), Some(&Value::Null));
        assert_eq!(legacy["device"]["battery_level"], json!(42));

        server.deliver(
            connection,
            r#"{"type":"android_snapshot","snapshot":{"screenOff":true}}"#,
        );
        server.deliver(connection, r#"{"type":"android_snapshot","snapshot":{}}"#);
        let unknown = mobile_api(&server.state).await;
        assert_eq!(unknown["status"]["is_online"], json!(true));
        assert_eq!(unknown["status"].get("screen_off"), Some(&Value::Null));

        server.state
            .add_client(ClientKind::DesktopReporter, Some("Desktop".to_string()), None)
            .expect("accept desktop connection");
        let desktop = serde_json::to_value(server.state.desktop_share_snapshot()).unwrap();
        assert_eq!(desktop["status"]["is_online"], json!(true));
        assert_eq!(desktop["status"].get("screen_off"), Some(&Value::Null));
    }
}
