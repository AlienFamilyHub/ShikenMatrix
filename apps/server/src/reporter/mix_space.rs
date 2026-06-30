use super::s3;
use super::types::{
    MediaMetadataData, PlaybackStateData, ReporterConfig, ReporterMessage, WindowInfoData,
};
use crate::state::SharedDashboardState;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Debug, Serialize)]
struct MixSpacePayload {
    media: Option<MixSpaceMediaPayload>,
    key: String,
    timestamp: u64,
    process: MixSpaceProcessPayload,
}

#[derive(Debug, Serialize)]
struct MixSpaceMediaPayload {
    artist: Option<String>,
    title: Option<String>,
    duration: f64,
    #[serde(rename = "elapsedTime")]
    elapsed_time: f64,
    #[serde(rename = "processName")]
    process_name: Option<String>,
    icon: Option<String>,
}

#[derive(Debug, Serialize)]
struct MixSpaceProcessPayload {
    #[serde(rename = "iconBase64")]
    icon_base64: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
    description: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingReport {
    window: Option<WindowInfoData>,
    media: Option<(MediaMetadataData, PlaybackStateData)>,
}

pub async fn run_mix_space_reporter(
    config: ReporterConfig,
    mut reporter_rx: mpsc::UnboundedReceiver<ReporterMessage>,
    state: SharedDashboardState,
) {
    let client = reqwest::Client::new();
    let mut pending = PendingReport {
        window: None,
        media: None,
    };
    let mut icon_urls = HashMap::new();
    let mut media_icon_urls = HashMap::new();

    while let Some(message) = reporter_rx.recv().await {
        match message {
            ReporterMessage::WindowInfo(window_message) => {
                pending.window = Some(window_message.data);
                send_mix_space_payload(
                    &client,
                    &config,
                    &pending,
                    &mut icon_urls,
                    &media_icon_urls,
                    &state,
                )
                .await;
            }
            ReporterMessage::MediaPlayback(media_message) => {
                pending.media = Some((media_message.metadata, media_message.playback_state));
                send_mix_space_payload(
                    &client,
                    &config,
                    &pending,
                    &mut icon_urls,
                    &media_icon_urls,
                    &state,
                )
                .await;
            }
            ReporterMessage::UploadArtwork {
                content_item_identifier,
                artwork_data,
                mime_type,
            } => {
                upload_media_icon(
                    &client,
                    &config,
                    content_item_identifier,
                    artwork_data,
                    mime_type,
                    &mut media_icon_urls,
                )
                .await;
                send_mix_space_payload(
                    &client,
                    &config,
                    &pending,
                    &mut icon_urls,
                    &media_icon_urls,
                    &state,
                )
                .await;
            }
            ReporterMessage::Shutdown => break,
        }
    }
}

async fn send_mix_space_payload(
    client: &reqwest::Client,
    config: &ReporterConfig,
    pending: &PendingReport,
    icon_urls: &mut HashMap<String, String>,
    media_icon_urls: &HashMap<String, String>,
    state: &SharedDashboardState,
) {
    let Some(window) = pending.window.as_ref() else {
        return;
    };

    if config.mix_space.endpoint.trim().is_empty() {
        warn!("Mix-Space endpoint is empty");
        return;
    }

    let icon_url = resolve_app_icon_url(client, config, window, icon_urls).await;
    let media_icon_url = pending
        .media
        .as_ref()
        .and_then(|(metadata, _)| metadata.content_item_identifier.as_ref())
        .and_then(|content_id| media_icon_urls.get(content_id))
        .cloned();
    let payload = build_payload(
        config,
        window,
        icon_url,
        media_icon_url,
        pending.media.as_ref(),
    );
    let method = config
        .mix_space
        .method
        .parse::<reqwest::Method>()
        .unwrap_or(reqwest::Method::POST);

    let result = client
        .request(method, config.mix_space.endpoint.trim())
        .json(&payload)
        .send()
        .await
        .and_then(|response| response.error_for_status());

    if let Err(error) = result {
        warn!("Mix-Space report failed: {error}");
        state.record_upstream_error();
    }
}

fn build_payload(
    config: &ReporterConfig,
    window: &WindowInfoData,
    icon_url: Option<String>,
    media_icon_url: Option<String>,
    media: Option<&(MediaMetadataData, PlaybackStateData)>,
) -> MixSpacePayload {
    let media = media.and_then(|(metadata, playback_state)| {
        if !playback_state.playing {
            return None;
        }

        Some(MixSpaceMediaPayload {
            artist: metadata.artist.clone(),
            title: metadata.title.clone(),
            duration: metadata.duration,
            elapsed_time: playback_state.elapsed_time,
            process_name: metadata.bundle_identifier.clone(),
            icon: media_icon_url.clone(),
        })
    });

    MixSpacePayload {
        media,
        key: config.mix_space.token.clone(),
        timestamp: current_timestamp(),
        process: MixSpaceProcessPayload {
            icon_base64: if icon_url.is_some() {
                None
            } else {
                window.icon_base64.clone()
            },
            icon_url: icon_url.or_else(|| window.icon_url.clone()),
            description: window_title_description(window),
            name: Some(window.process_name.clone()),
        },
    }
}

async fn resolve_app_icon_url(
    client: &reqwest::Client,
    config: &ReporterConfig,
    window: &WindowInfoData,
    icon_urls: &mut HashMap<String, String>,
) -> Option<String> {
    if !config.s3.enabled {
        return None;
    }

    let cache_key = window
        .app_id
        .clone()
        .unwrap_or_else(|| window.process_name.clone());
    if let Some(url) = icon_urls.get(&cache_key) {
        return Some(url.clone());
    }

    let icon_base64 = window.icon_base64.as_ref()?;
    let icon_data = match BASE64.decode(icon_base64) {
        Ok(data) => data,
        Err(error) => {
            warn!("Decode app icon failed: {error}");
            return None;
        }
    };

    match s3::upload_app_icon(client, config, &icon_data, &window.process_name).await {
        Ok(url) => {
            icon_urls.insert(cache_key, url.clone());
            Some(url)
        }
        Err(error) => {
            warn!("S3 app icon upload failed: {error}");
            None
        }
    }
}

async fn upload_media_icon(
    client: &reqwest::Client,
    config: &ReporterConfig,
    content_item_identifier: String,
    artwork_data: Vec<u8>,
    mime_type: String,
    media_icon_urls: &mut HashMap<String, String>,
) {
    if media_icon_urls.contains_key(&content_item_identifier) || !config.s3.enabled {
        return;
    }

    match s3::upload_media_icon(client, config, &artwork_data, &mime_type).await {
        Ok(url) => {
            media_icon_urls.insert(content_item_identifier, url);
        }
        Err(error) => warn!("S3 media icon upload failed: {error}"),
    }
}

fn window_title_description(window: &WindowInfoData) -> Option<String> {
    if window.title.trim().is_empty() {
        None
    } else {
        Some(window.title.clone())
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
