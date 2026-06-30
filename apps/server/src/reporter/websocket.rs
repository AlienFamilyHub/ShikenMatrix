use super::types::{
    MediaPlaybackMessage, ReporterConfig, ReporterMessage, ServerMessage, WindowInfoMessage,
    build_native_websocket_url,
};
use crate::state::SharedDashboardState;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::Message};
use tracing::{info, warn};

pub async fn run_native_reporter(
    config: ReporterConfig,
    mut reporter_rx: mpsc::UnboundedReceiver<ReporterMessage>,
    server_tx: mpsc::UnboundedSender<String>,
    state: SharedDashboardState,
) {
    let mut reconnect_attempts = 0;

    loop {
        if connect_once(&config, &mut reporter_rx, &server_tx, &state).await {
            reconnect_attempts = 0;
        } else {
            reconnect_attempts += 1;
            wait_before_reconnect(reconnect_attempts, &mut reconnect_attempts).await;
        }
    }
}

async fn connect_once(
    config: &ReporterConfig,
    reporter_rx: &mut mpsc::UnboundedReceiver<ReporterMessage>,
    server_tx: &mpsc::UnboundedSender<String>,
    state: &SharedDashboardState,
) -> bool {
    let ws_url = match build_native_websocket_url(config) {
        Ok(url) => url,
        Err(error) => {
            warn!("invalid upstream websocket url: {error}");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            return false;
        }
    };

    let connector = Connector::Rustls(std::sync::Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::from_iter(
                webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
            ))
            .with_no_client_auth(),
    ));

    let connect_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(15),
        connect_async_tls_with_config(ws_url.as_str(), None, false, Some(connector)),
    )
    .await;

    match connect_result {
        Ok(Ok((ws_stream, response))) => {
            info!("upstream websocket connected: {}", response.status());
            state.inc_native_upstream();
            run_socket_loop(ws_stream, reporter_rx, server_tx, state).await;
            state.dec_native_upstream();
            false
        }
        Ok(Err(error)) => {
            warn!("upstream websocket connection failed: {error}");
            state.record_upstream_error();
            false
        }
        Err(_) => {
            warn!("upstream websocket connection timeout");
            state.record_upstream_error();
            false
        }
    }
}

async fn run_socket_loop<S>(
    ws_stream: S,
    reporter_rx: &mut mpsc::UnboundedReceiver<ReporterMessage>,
    server_tx: &mpsc::UnboundedSender<String>,
    state: &SharedDashboardState,
) where
    S: futures_util::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
        + futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    let (mut upstream_writer, mut upstream_reader) = ws_stream.split();

    loop {
        tokio::select! {
            Some(message) = reporter_rx.recv() => {
                if handle_outgoing(message, &mut upstream_writer, state).await {
                    return;
                }
            }
            Some(message) = upstream_reader.next() => {
                if handle_incoming(message, server_tx) {
                    break;
                }
            }
        }
    }
}

async fn handle_outgoing<W>(
    message: ReporterMessage,
    upstream_writer: &mut W,
    state: &SharedDashboardState,
) -> bool
where
    W: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    match message {
        ReporterMessage::WindowInfo(window_message) => {
            send_json(
                upstream_writer,
                &OutgoingWindowInfoMessage::from(window_message),
                state,
            )
            .await
        }
        ReporterMessage::MediaPlayback(media_message) => {
            send_json(
                upstream_writer,
                &OutgoingMediaPlaybackMessage::from(media_message),
                state,
            )
            .await
        }
        ReporterMessage::UploadArtwork {
            content_item_identifier,
            artwork_data,
            mime_type,
        } => {
            upload_artwork(
                upstream_writer,
                content_item_identifier,
                artwork_data,
                mime_type,
                state,
            )
            .await
        }
        ReporterMessage::Shutdown => {
            let _ = upstream_writer.close().await;
            true
        }
    }
}

async fn send_json<W, T>(upstream_writer: &mut W, message: &T, state: &SharedDashboardState) -> bool
where
    W: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
    T: serde::Serialize,
{
    let Ok(json) = serde_json::to_string(message) else {
        return false;
    };

    let failed = upstream_writer
        .send(Message::Text(json.into()))
        .await
        .is_err();
    if failed {
        state.record_upstream_error();
    }
    failed
}

async fn upload_artwork<W>(
    upstream_writer: &mut W,
    content_item_identifier: String,
    artwork_data: Vec<u8>,
    mime_type: String,
    state: &SharedDashboardState,
) -> bool
where
    W: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    let meta_message = OutgoingUploadArtworkMetaMessage {
        msg_type: "upload_artwork_meta",
        content_item_identifier,
        mime_type,
    };

    let Ok(meta_json) = serde_json::to_string(&meta_message) else {
        return false;
    };

    let meta_failed = upstream_writer
        .send(Message::Text(meta_json.into()))
        .await
        .is_err();
    if meta_failed {
        state.record_upstream_error();
        return true;
    }

    let binary_failed = upstream_writer
        .send(Message::Binary(artwork_data.into()))
        .await
        .is_err();
    if binary_failed {
        state.record_upstream_error();
    }
    binary_failed
}

fn handle_incoming(
    message: Result<Message, tokio_tungstenite::tungstenite::Error>,
    server_tx: &mpsc::UnboundedSender<String>,
) -> bool {
    match message {
        Ok(Message::Text(text)) => {
            forward_artwork_message(text.as_str(), server_tx);
            false
        }
        Ok(Message::Close(_)) => true,
        Err(error) => {
            warn!("upstream websocket error: {error}");
            true
        }
        _ => false,
    }
}

fn forward_artwork_message(text: &str, server_tx: &mpsc::UnboundedSender<String>) {
    let Ok(server_message) = serde_json::from_str::<ServerMessage>(text) else {
        return;
    };

    if server_message.msg_type != "artwork_uploaded"
        || server_message.content_item_identifier.is_none()
        || server_message.artwork_url.is_none()
    {
        return;
    }

    let _ = server_tx.send(text.to_string());
}

async fn wait_before_reconnect(attempts: u32, reconnect_attempts: &mut u32) {
    if attempts >= 5 {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        *reconnect_attempts = 0;
    } else {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}

#[derive(serde::Serialize)]
struct OutgoingWindowInfoMessage {
    #[serde(rename = "type")]
    msg_type: &'static str,
    data: super::types::WindowInfoData,
}

impl From<WindowInfoMessage> for OutgoingWindowInfoMessage {
    fn from(message: WindowInfoMessage) -> Self {
        Self {
            msg_type: "window_info",
            data: message.data,
        }
    }
}

#[derive(serde::Serialize)]
struct OutgoingMediaPlaybackMessage {
    #[serde(rename = "type")]
    msg_type: &'static str,
    metadata: super::types::MediaMetadataData,
    playback_state: super::types::PlaybackStateData,
}

impl From<MediaPlaybackMessage> for OutgoingMediaPlaybackMessage {
    fn from(message: MediaPlaybackMessage) -> Self {
        Self {
            msg_type: "media_playback",
            metadata: message.metadata,
            playback_state: message.playback_state,
        }
    }
}

#[derive(serde::Serialize)]
struct OutgoingUploadArtworkMetaMessage {
    #[serde(rename = "type")]
    msg_type: &'static str,
    content_item_identifier: String,
    mime_type: String,
}
