use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterConfig {
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

#[derive(Debug, Clone, Serialize)]
pub enum ReporterEvent {
    Log {
        level: LogLevel,
        message: String,
    },
    WindowUpdated {
        title: String,
        process_name: String,
        pid: u32,
        icon_data: Option<Vec<u8>>,
    },
    MediaUpdated {
        title: String,
        artist: String,
        album: String,
        duration: f64,
        elapsed_time: f64,
        playing: bool,
        artwork_data: Option<Vec<u8>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum ReporterMessage {
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
pub(super) struct ServerMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub content_item_identifier: Option<String>,
    #[serde(default)]
    pub artwork_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct WindowInfoMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: WindowInfoData,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct MediaPlaybackMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub metadata: MediaMetadataData,
    pub playback_state: PlaybackStateData,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct UploadArtworkMetaMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub content_item_identifier: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash)]
pub(super) struct WindowInfoData {
    pub title: String,
    pub process_name: String,
    #[serde(skip_serializing)]
    pub icon_base64: Option<String>,
    pub icon_url: Option<String>,
    pub app_id: Option<String>,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(super) struct MediaMetadataData {
    pub bundle_identifier: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub artwork_url: Option<String>,
    pub content_item_identifier: Option<String>,
}

impl Hash for MediaMetadataData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bundle_identifier.hash(state);
        self.title.hash(state);
        self.artist.hash(state);
        self.album.hash(state);
        ((self.duration * 1000.0) as i64).hash(state);
        self.artwork_url.hash(state);
        self.content_item_identifier.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(super) struct PlaybackStateData {
    pub playing: bool,
    pub playback_rate: f64,
    pub elapsed_time: f64,
}

impl Hash for PlaybackStateData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.playing.hash(state);
        ((self.playback_rate * 100.0) as i64).hash(state);
        (self.elapsed_time as i64).hash(state);
    }
}

fn default_mix_space_method() -> String {
    "POST".to_string()
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

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn default_s3_key_template() -> String {
    "{kind}/{Y}/{M}/{D}/{SHA}.{ext}".to_string()
}

pub(super) fn compute_hash<T: Hash>(data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn build_websocket_url(config: &ReporterConfig) -> Result<Url, url::ParseError> {
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
