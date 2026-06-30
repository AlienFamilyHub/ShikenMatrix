use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReporterConfig {
    #[serde(default)]
    pub server: ServerReporterConfig,
    #[serde(default)]
    pub protocol: ReporterProtocol,
    #[serde(default)]
    pub enable_media_reporting: bool,
    #[serde(default)]
    pub native: NativeReporterConfig,
    #[serde(default)]
    pub mix_space: MixSpaceReporterConfig,
    #[serde(default)]
    pub s3: S3ReporterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerReporterConfig {
    #[serde(default = "default_server_ws_url")]
    pub ws_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NativeReporterConfig {
    #[serde(default)]
    pub ws_url: String,
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixSpaceReporterConfig {
    #[serde(default)]
    pub endpoint: String,
    #[serde(default = "default_mix_space_method")]
    pub method: String,
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3ReporterConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bucket: String,
    #[serde(default = "default_s3_region")]
    pub region: String,
    #[serde(default)]
    pub access_key: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub custom_domain: String,
    #[serde(default = "default_s3_key_template")]
    pub key_template: String,
    #[serde(default)]
    pub lifecycle_days: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReporterProtocol {
    #[default]
    Native,
    MixSpace,
}

#[derive(Debug, Clone)]
pub enum ReporterMessage {
    WindowInfo(WindowInfoMessage),
    MediaPlayback(MediaPlaybackMessage),
    UploadArtwork {
        content_item_identifier: String,
        artwork_data: Vec<u8>,
        mime_type: String,
    },
    Shutdown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub content_item_identifier: Option<String>,
    #[serde(default)]
    pub artwork_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfoMessage {
    pub data: WindowInfoData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPlaybackMessage {
    pub metadata: MediaMetadataData,
    pub playback_state: PlaybackStateData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadArtworkMetaMessage {
    pub content_item_identifier: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct WindowInfoData {
    pub title: String,
    pub process_name: String,
    pub icon_base64: Option<String>,
    pub icon_url: Option<String>,
    pub app_id: Option<String>,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaMetadataData {
    pub bundle_identifier: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub artwork_url: Option<String>,
    pub content_item_identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaybackStateData {
    pub playing: bool,
    pub playback_rate: f64,
    pub elapsed_time: f64,
}

impl Default for ServerReporterConfig {
    fn default() -> Self {
        Self {
            ws_url: default_server_ws_url(),
        }
    }
}

impl Default for MixSpaceReporterConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            method: default_mix_space_method(),
            token: String::new(),
        }
    }
}

impl Default for S3ReporterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bucket: String::new(),
            region: default_s3_region(),
            access_key: String::new(),
            secret_key: String::new(),
            endpoint: String::new(),
            custom_domain: String::new(),
            key_template: default_s3_key_template(),
            lifecycle_days: 0,
        }
    }
}

fn default_server_ws_url() -> String {
    "ws://127.0.0.1:4317/reporter".to_string()
}

fn default_mix_space_method() -> String {
    "POST".to_string()
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn default_s3_key_template() -> String {
    "{kind}/{Y}/{M}/{D}/{SHA}.{ext}".to_string()
}

pub fn build_native_websocket_url(config: &ReporterConfig) -> Result<Url, url::ParseError> {
    let ws_url = config
        .native
        .ws_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let mut url = Url::parse(&ws_url)?;
    url.query_pairs_mut()
        .append_pair("token", &config.native.token);
    Ok(url)
}
