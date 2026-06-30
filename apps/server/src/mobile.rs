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
                                forward_mobile_payload(payload, reporter_tx.as_ref(), &state, client_id);
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
    state.record_window(client_id, ClientKind::Mobile, &window_data);

    if let Some(reporter_tx) = reporter_tx {
        let _ = reporter_tx.send(ReporterMessage::WindowInfo(WindowInfoMessage {
            data: window_data,
        }));
    }

    if let Some(media) = payload.snapshot.media {
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
        state.record_media(client_id, ClientKind::Mobile, &metadata, &playback);
        if let Some(reporter_tx) = reporter_tx {
            let _ = reporter_tx.send(ReporterMessage::MediaPlayback(MediaPlaybackMessage {
                metadata,
                playback_state: playback,
            }));
        }
    }

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
    state.record_mobile_device(device);
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
