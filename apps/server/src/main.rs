mod mobile;
mod panel;
mod reporter;
mod state;
mod storage;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{get, post, delete};
use futures_util::{SinkExt, StreamExt};
use reporter::{
    ReporterMessage, ReporterProtocol, UploadArtworkMetaMessage, run_mix_space_reporter,
    run_native_reporter,
};
use state::{ClientKind, DashboardState, SharedDashboardState};
use std::env;
use std::sync::Arc;
use storage::Storage;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let bind_addr =
        env::var("SHIKENMATRIX_SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:4317".to_string());
    let db_path =
        env::var("SHIKENMATRIX_DB_PATH").unwrap_or_else(|_| "shikenmatrix.sqlite3".to_string());
    let storage = Storage::open(db_path)?;
    let state: SharedDashboardState = Arc::new(DashboardState::new(bind_addr.clone(), storage));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/login", post(panel::api_login))
        .route("/api/share", get(panel::api_share))
        .route("/api/state", get(panel::api_state))
        .route("/api/upstream", post(panel::api_save_upstream))
        .route("/api/clients/keys", get(panel::api_get_client_keys).post(panel::api_create_client_key))
        .route("/api/clients/keys/{id}", delete(panel::api_delete_client_key))
        .route("/api/health", get(panel::api_health))
        .route("/reporter", get(reporter_ws))
        .route("/mobile", get(mobile::mobile_ws))
        .fallback(panel::panel_fallback)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!("ShikenMatrix server listening on {bind_addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

#[derive(serde::Deserialize)]
struct ReporterQuery {
    key: Option<String>,
    #[serde(default)]
    client: Option<String>,
    #[serde(default, rename = "deviceId")]
    device_id: Option<String>,
}

async fn reporter_ws(
    State(state): State<SharedDashboardState>,
    axum::extract::Query(query): axum::extract::Query<ReporterQuery>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    let key = query.key.unwrap_or_default();
    if !state.storage().verify_client_key(&key) {
        return (axum::http::StatusCode::UNAUTHORIZED, "invalid client key").into_response();
    }
    let session = ReporterSession {
        client: query.client,
        device_id: query.device_id,
    };
    upgrade.on_upgrade(move |socket| handle_reporter_socket(socket, state, session))
}

struct ReporterSession {
    client: Option<String>,
    device_id: Option<String>,
}

async fn handle_reporter_socket(
    socket: WebSocket,
    state: SharedDashboardState,
    session: ReporterSession,
) {
    let (mut client_sender, mut client_receiver) = socket.split();

    let Some((client_id, session_token)) =
        state.add_client(ClientKind::DesktopReporter, session.client, session.device_id)
    else {
        let _ = client_sender
            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                code: 1008,
                reason: "desktop reporter already connected".into(),
            })))
            .await;
        return;
    };

    let (server_tx, mut server_rx) = mpsc::unbounded_channel();
    let (reporter_tx, reporter_task) = start_upstream_reporter(&state, server_tx);

    let mut pending_artwork_meta: Option<UploadArtworkMetaMessage> = None;

    // 心跳：每 20s 发 Ping，超过 60s 未收到任何帧则关闭半开连接
    let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_secs(20));
    heartbeat.reset();
    let mut last_seen = std::time::Instant::now();
    let idle_timeout = tokio::time::Duration::from_secs(60);

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if client_sender.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
                if last_seen.elapsed() > idle_timeout {
                    warn!("reporter client idle timeout, closing");
                    let _ = client_sender
                        .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                            code: 1001,
                            reason: "idle timeout".into(),
                        })))
                        .await;
                    break;
                }
            }
            Some(server_message) = server_rx.recv() => {
                if client_sender.send(Message::Text(server_message.into())).await.is_err() {
                    break;
                }
            }
            Some(client_message) = client_receiver.next() => {
                match client_message {
                    Ok(Message::Pong(_)) | Ok(Message::Ping(_)) => {
                        last_seen = std::time::Instant::now();
                    }
                    Ok(Message::Close(_)) => break,
                    Err(error) => {
                        error!("reporter client websocket error: {error}");
                        break;
                    }
                    Ok(other) => {
                        last_seen = std::time::Instant::now();
                        if handle_client_message(
                            Ok(other),
                            reporter_tx.as_ref(),
                            &mut pending_artwork_meta,
                            &state,
                            client_id,
                        )
                        .await
                        {
                            break;
                        }
                    }
                }
            }
            else => break,
        }
    }

    state.remove_client(client_id, session_token, ClientKind::DesktopReporter);
    if let Some(reporter_tx) = reporter_tx {
        let _ = reporter_tx.send(ReporterMessage::Shutdown);
    }
    if let Some(reporter_task) = reporter_task {
        reporter_task.abort();
    }
}

async fn handle_client_message(
    message: Result<Message, axum::Error>,
    reporter_tx: Option<&mpsc::UnboundedSender<ReporterMessage>>,
    pending_artwork_meta: &mut Option<UploadArtworkMetaMessage>,
    state: &SharedDashboardState,
    client_id: u64,
) -> bool {
    match message {
        Ok(Message::Text(text)) => handle_client_text(
            text.as_str(),
            reporter_tx,
            pending_artwork_meta,
            state,
            client_id,
        ),
        Ok(Message::Binary(data)) => {
            if let Some(meta) = pending_artwork_meta.take() {
                state.record_artwork_upload(
                    client_id,
                    ClientKind::DesktopReporter,
                    &meta.content_item_identifier,
                );
                if let Some(reporter_tx) = reporter_tx {
                    let _ = reporter_tx.send(ReporterMessage::UploadArtwork {
                        content_item_identifier: meta.content_item_identifier,
                        artwork_data: data.to_vec(),
                        mime_type: meta.mime_type,
                    });
                }
            }
            false
        }
        Ok(Message::Close(_)) => true,
        Err(error) => {
            error!("reporter client websocket error: {error}");
            true
        }
        _ => false,
    }
}

fn handle_client_text(
    text: &str,
    reporter_tx: Option<&mpsc::UnboundedSender<ReporterMessage>>,
    pending_artwork_meta: &mut Option<UploadArtworkMetaMessage>,
    state: &SharedDashboardState,
    client_id: u64,
) -> bool {
    match serde_json::from_str::<ReporterMessageEnvelope>(text) {
        Ok(ReporterMessageEnvelope::WindowInfo(message)) => {
            state.record_window_info(
                client_id,
                ClientKind::DesktopReporter,
                &message.data.title,
                &message.data.process_name,
            );
            if let Some(reporter_tx) = reporter_tx {
                let _ = reporter_tx.send(ReporterMessage::WindowInfo(message));
            }
        }
        Ok(ReporterMessageEnvelope::MediaPlayback(message)) => {
            state.record_media_playback(
                client_id,
                ClientKind::DesktopReporter,
                message.metadata.title.as_deref(),
                message.metadata.artist.as_deref(),
            );
            if let Some(reporter_tx) = reporter_tx {
                let _ = reporter_tx.send(ReporterMessage::MediaPlayback(message));
            }
        }
        Ok(ReporterMessageEnvelope::UploadArtworkMeta(message)) => {
            *pending_artwork_meta = Some(message);
        }
        Ok(ReporterMessageEnvelope::Shutdown) => return true,
        Err(error) => warn!("invalid reporter client message: {error}"),
    }

    false
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

fn start_upstream_reporter(
    state: &SharedDashboardState,
    server_tx: mpsc::UnboundedSender<String>,
) -> (
    Option<mpsc::UnboundedSender<ReporterMessage>>,
    Option<tokio::task::JoinHandle<()>>,
) {
    let Some(config) = state.upstream_settings().to_reporter_config() else {
        return (None, None);
    };
    let (reporter_tx, reporter_rx) = mpsc::unbounded_channel();
    let reporter_task = match config.protocol {
        ReporterProtocol::Native => tokio::spawn(run_native_reporter(
            config,
            reporter_rx,
            server_tx,
            state.clone(),
        )),
        ReporterProtocol::MixSpace => {
            tokio::spawn(run_mix_space_reporter(config, reporter_rx, state.clone()))
        }
    };
    (Some(reporter_tx), Some(reporter_task))
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum ReporterMessageEnvelope {
    #[serde(rename = "window_info")]
    WindowInfo(reporter::WindowInfoMessage),
    #[serde(rename = "media_playback")]
    MediaPlayback(reporter::MediaPlaybackMessage),
    #[serde(rename = "upload_artwork_meta")]
    UploadArtworkMeta(UploadArtworkMetaMessage),
    #[serde(rename = "shutdown")]
    Shutdown,
}
