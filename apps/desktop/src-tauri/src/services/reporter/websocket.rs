use super::Reporter;
use super::types::{
    ReporterConfig, ReporterMessage, ServerMessage, UploadArtworkMetaMessage,
    build_server_websocket_url,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc as tokio_mpsc;
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::Message};
use tracing::info;

impl Reporter {
    pub(super) async fn run_reporter(
        config: Arc<RwLock<ReporterConfig>>,
        mut rx: tokio_mpsc::UnboundedReceiver<ReporterMessage>,
        artwork_urls: Arc<RwLock<HashMap<String, String>>>,
        is_connected: Arc<AtomicBool>,
        is_running: Arc<AtomicBool>,
        last_error: Arc<RwLock<Option<String>>>,
    ) {
        let mut reconnect_attempts = 0;
        while is_running.load(Ordering::Relaxed) {
            let cfg = config.read().unwrap().clone();

            if connect_once(&cfg, &mut rx, &artwork_urls, &is_connected, &last_error).await {
                reconnect_attempts = 0;
            } else {
                reconnect_attempts += 1;
                wait_before_reconnect(reconnect_attempts, &mut reconnect_attempts).await;
            }
        }
    }
}

async fn connect_once(
    config: &ReporterConfig,
    rx: &mut tokio_mpsc::UnboundedReceiver<ReporterMessage>,
    artwork_urls: &Arc<RwLock<HashMap<String, String>>>,
    is_connected: &Arc<AtomicBool>,
    last_error: &Arc<RwLock<Option<String>>>,
) -> bool {
    let ws_url = match build_server_websocket_url(config) {
        Ok(url) => url,
        Err(error) => {
            Reporter::set_last_error(last_error, format!("Invalid server WebSocket URL: {error}"));
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            return false;
        }
    };

    info!("Connecting to ShikenMatrix server WebSocket: {ws_url}");
    is_connected.store(false, Ordering::Relaxed);

    let connector = Connector::Rustls(Arc::new(
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
            info!("WebSocket connected: {}", response.status());
            Reporter::clear_last_error(last_error);
            is_connected.store(true, Ordering::Relaxed);
            run_socket_loop(ws_stream, rx, artwork_urls, is_connected, last_error).await;
            false
        }
        Ok(Err(error)) => {
            is_connected.store(false, Ordering::Relaxed);
            Reporter::set_last_error(last_error, format!("WebSocket connection failed: {error}"));
            false
        }
        Err(_) => {
            is_connected.store(false, Ordering::Relaxed);
            Reporter::set_last_error(last_error, "WebSocket connection timeout");
            false
        }
    }
}

async fn run_socket_loop<S>(
    ws_stream: S,
    rx: &mut tokio_mpsc::UnboundedReceiver<ReporterMessage>,
    artwork_urls: &Arc<RwLock<HashMap<String, String>>>,
    is_connected: &Arc<AtomicBool>,
    last_error: &Arc<RwLock<Option<String>>>,
) where
    S: futures_util::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
        + futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    let (mut write, mut read) = ws_stream.split();

    loop {
        tokio::select! {
            Some(message) = rx.recv() => {
                if handle_outgoing(message, &mut write, is_connected, last_error).await {
                    return;
                }
            }
            Some(message) = read.next() => {
                if handle_incoming(message, artwork_urls, last_error) {
                    break;
                }
            }
        }
    }

    is_connected.store(false, Ordering::Relaxed);
}

async fn handle_outgoing<W>(
    message: ReporterMessage,
    write: &mut W,
    is_connected: &Arc<AtomicBool>,
    last_error: &Arc<RwLock<Option<String>>>,
) -> bool
where
    W: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    match message {
        ReporterMessage::WindowInfo(window_message) => {
            send_json(write, &window_message, "window", last_error).await
        }
        ReporterMessage::MediaPlayback(media_message) => {
            send_json(write, &media_message, "media", last_error).await
        }
        ReporterMessage::UploadArtwork {
            content_item_identifier,
            artwork_data,
            mime_type,
        } => {
            upload_artwork(
                write,
                content_item_identifier,
                artwork_data,
                mime_type,
                last_error,
            )
            .await
        }
        ReporterMessage::Shutdown => {
            is_connected.store(false, Ordering::Relaxed);
            let _ = write.close().await;
            true
        }
    }
}

async fn send_json<W, T>(
    write: &mut W,
    message: &T,
    label: &str,
    last_error: &Arc<RwLock<Option<String>>>,
) -> bool
where
    W: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
    T: serde::Serialize,
{
    if let Ok(json) = serde_json::to_string(message) {
        if let Err(error) = write.send(Message::Text(json.into())).await {
            Reporter::set_last_error(
                last_error,
                format!("Failed to send {label} message: {error}"),
            );
            return true;
        }
    }
    false
}

async fn upload_artwork<W>(
    write: &mut W,
    content_item_identifier: String,
    artwork_data: Vec<u8>,
    mime_type: String,
    last_error: &Arc<RwLock<Option<String>>>,
) -> bool
where
    W: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    let meta_message = UploadArtworkMetaMessage {
        msg_type: "upload_artwork_meta".to_string(),
        content_item_identifier: content_item_identifier.clone(),
        mime_type,
    };

    let Ok(meta_json) = serde_json::to_string(&meta_message) else {
        return false;
    };

    let sent_meta = write.send(Message::Text(meta_json.into())).await;
    let sent_binary = write.send(Message::Binary(artwork_data.into())).await;
    if sent_meta.is_err() || sent_binary.is_err() {
        Reporter::set_last_error(
            last_error,
            format!("Failed to upload artwork: {content_item_identifier}"),
        );
        return true;
    }
    false
}

fn handle_incoming(
    message: Result<Message, tokio_tungstenite::tungstenite::Error>,
    artwork_urls: &Arc<RwLock<HashMap<String, String>>>,
    last_error: &Arc<RwLock<Option<String>>>,
) -> bool {
    match message {
        Ok(Message::Text(text)) => {
            info!("Received: {text}");
            cache_artwork_url(&text, artwork_urls);
            false
        }
        Ok(Message::Close(_)) => {
            Reporter::set_last_error(last_error, "WebSocket closed by server");
            true
        }
        Err(error) => {
            Reporter::set_last_error(last_error, format!("WebSocket error: {error}"));
            true
        }
        _ => false,
    }
}

/// Maximum number of artwork URLs to cache.
/// Prevents unbounded HashMap growth when cycling through many tracks.
const MAX_ARTWORK_CACHE: usize = 100;

fn cache_artwork_url(text: &str, artwork_urls: &Arc<RwLock<HashMap<String, String>>>) {
    let Ok(server_message) = serde_json::from_str::<ServerMessage>(text) else {
        return;
    };

    if server_message.msg_type != "artwork_uploaded" {
        return;
    }

    if let (Some(content_id), Some(url)) = (
        server_message.content_item_identifier,
        server_message.artwork_url,
    ) {
        if let Ok(mut urls) = artwork_urls.write() {
            // Evict oldest entries when cache is full
            if urls.len() >= MAX_ARTWORK_CACHE {
                let keys: Vec<String> = urls
                    .keys()
                    .take(urls.len() - MAX_ARTWORK_CACHE + 1)
                    .cloned()
                    .collect();
                for key in keys {
                    urls.remove(&key);
                }
            }
            urls.insert(content_id, url);
        }
    }
}

async fn wait_before_reconnect(attempts: u32, reconnect_attempts: &mut u32) {
    if attempts >= 5 {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        *reconnect_attempts = 0;
    } else {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
