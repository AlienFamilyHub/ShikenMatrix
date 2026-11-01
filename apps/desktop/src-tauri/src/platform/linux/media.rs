//! Native session-bus MPRIS snapshots. Call from the reporter's blocking thread.

use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::time::{Duration, Instant};

use mpris::{DBusError, Metadata, MetadataValue, PlaybackStatus, Player, PlayerFinder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaMetadata {
    pub bundle_identifier: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub artwork_data: Option<Vec<u8>>,
    pub artwork_mime_type: Option<String>,
    pub content_item_identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaybackState {
    pub playing: bool,
    pub playback_rate: f64,
    pub elapsed_time: f64,
}

const MAX_ARTWORK_BYTES: u64 = 8 * 1024 * 1024;
const PLAYER_TIMEOUT_MS: i32 = 100;
const DISCOVERY_BUDGET: Duration = Duration::from_secs(4);
const ARTWORK_TIMEOUT: Duration = Duration::from_secs(2);
const ARTWORK_RETRY: Duration = Duration::from_secs(30);

#[derive(Clone)]
struct Artwork {
    bytes: Vec<u8>,
    mime: String,
}

struct CachedArtwork {
    key: (String, String),
    value: Option<Artwork>,
    attempted: Instant,
}

#[derive(Default)]
struct MediaContext {
    selected_bus: Option<String>,
    artwork: Option<CachedArtwork>,
    worker: Option<ArtworkWorker>,
}

struct ArtworkWorker {
    requests: SyncSender<(String, String)>,
    results: Receiver<CachedArtwork>,
    pending: bool,
}

// Neither mpris's Rc-backed connection nor its player objects cross threads.
// The current artwork and HTTP connection pool survive polls without a global
// mutex held during D-Bus, file or network operations.
thread_local! {
    static CONTEXT: RefCell<MediaContext> = RefCell::new(MediaContext::default());
}

/// Return metadata and state from one selected player, never two independent
/// discoveries. Playing outranks Paused; ties retain the previous selection,
/// then use the complete bus name for deterministic ordering.
pub fn get_media_info() -> Result<Option<(MediaMetadata, PlaybackState)>, String> {
    CONTEXT.with(|context| {
        let mut context = context
            .try_borrow_mut()
            .map_err(|_| "Reentrant Linux media polling is not supported".to_string())?;
        poll_media(&mut context)
    })
}

fn poll_media(
    context: &mut MediaContext,
) -> Result<Option<(MediaMetadata, PlaybackState)>, String> {
    let mut finder = PlayerFinder::new()
        .map_err(|error| format!("Cannot connect to the D-Bus user session for MPRIS: {error}"))?;
    finder.set_player_timeout_ms(PLAYER_TIMEOUT_MS);
    let players = finder.iter_players().map_err(|error| {
        format!("Cannot discover MPRIS players on the D-Bus user session: {error}")
    })?;
    let exceeds_player_limit = players.len() > 32;
    let deadline = Instant::now() + DISCOVERY_BUDGET;
    let mut candidates = Vec::new();
    // Bound hostile or hung player populations as well as individual calls.
    for result in players.take(32) {
        if Instant::now() >= deadline {
            break;
        }
        let Ok(player) = result else { continue };
        let Ok(status) = player.get_playback_status() else {
            continue;
        };
        if status_priority(status).is_some() {
            candidates.push((player, status));
        }
    }
    candidates.sort_by(|(left, left_status), (right, right_status)| {
        selection_key(
            left.bus_name(),
            *left_status,
            context.selected_bus.as_deref(),
        )
        .cmp(&selection_key(
            right.bus_name(),
            *right_status,
            context.selected_bus.as_deref(),
        ))
    });
    for (player, expected_status) in candidates {
        if Instant::now() >= deadline {
            break;
        }
        let Some((mut metadata, state, art_url)) = player_snapshot(&player, expected_status) else {
            continue;
        };
        context.selected_bus = Some(player.bus_name().to_owned());
        if let Some(art_url) = art_url {
            let key = (
                metadata.content_item_identifier.clone().unwrap_or_default(),
                art_url,
            );
            if let Some(artwork) = context.artwork_for(key) {
                metadata.artwork_data = Some(artwork.bytes);
                metadata.artwork_mime_type = Some(artwork.mime);
            }
        } else {
            context.artwork = None;
        }
        return Ok(Some((metadata, state)));
    }
    // A player may disappear or violate MPRIS without invalidating the session.
    // Recheck ListNames so a lost bus is not silently reported as "no media".
    finder
        .iter_players()
        .map_err(|error| format!("MPRIS discovery lost the D-Bus user session: {error}"))?;
    if exceeds_player_limit || Instant::now() >= deadline {
        return Err("MPRIS player inspection exceeded its bounded polling budget; a player may be unresponsive or more than 32 players are registered".into());
    }
    context.selected_bus = None;
    context.artwork = None;
    Ok(None)
}

fn status_priority(status: PlaybackStatus) -> Option<u8> {
    match status {
        PlaybackStatus::Playing => Some(0),
        PlaybackStatus::Paused => Some(1),
        PlaybackStatus::Stopped => None,
    }
}

fn selection_key<'a>(
    bus: &'a str,
    status: PlaybackStatus,
    previous: Option<&str>,
) -> (u8, bool, &'a str) {
    (
        status_priority(status).unwrap_or(2),
        previous != Some(bus),
        bus,
    )
}

fn optional_metadata(player: &Player) -> Result<Metadata, DBusError> {
    match player.get_metadata() {
        Ok(metadata) => Ok(metadata),
        Err(DBusError::TransportError(error))
            if matches!(
                error.name(),
                Some(
                    "org.freedesktop.DBus.Error.UnknownProperty"
                        | "org.freedesktop.DBus.Error.NotSupported"
                )
            ) =>
        {
            Ok(Metadata::default())
        }
        Err(error) => Err(error),
    }
}

fn player_snapshot(
    player: &Player,
    expected_status: PlaybackStatus,
) -> Option<(MediaMetadata, PlaybackState, Option<String>)> {
    let status = player.get_playback_status().ok()?;
    status_priority(status)?;
    if status != expected_status {
        return None;
    }
    let source = optional_metadata(player).ok()?;
    let position = player.get_position_in_microseconds().unwrap_or(0);
    let rate = player.get_playback_rate().unwrap_or(1.0);
    let desktop_entry = player
        .get_desktop_entry()
        .ok()
        .flatten()
        .filter(|entry| !entry.is_empty());
    // The crate exposes individual property reads, not atomic GetAll. Reject a
    // track/state transition during the read instead of mixing two tracks.
    let after = optional_metadata(player).ok()?;
    let track_changed = [
        "mpris:trackid",
        "xesam:url",
        "xesam:title",
        "xesam:artist",
        "xesam:album",
        "mpris:length",
        "mpris:artUrl",
    ]
    .iter()
    .any(|key| source.get(key) != after.get(key));
    if track_changed || player.get_playback_status().ok()? != status {
        return None;
    }
    let art_url = source
        .art_url()
        .filter(|url| !url.is_empty())
        .map(str::to_owned);
    let (metadata, state) = convert_snapshot(
        player.bus_name(),
        desktop_entry,
        &source,
        status,
        position,
        rate,
    );
    Some((metadata, state, art_url))
}

fn duration_microseconds(source: &Metadata) -> u64 {
    // mpris::Metadata::length() casts negative signed lengths to u64.
    match source.get("mpris:length") {
        Some(MetadataValue::I64(value)) => (*value).max(0) as u64,
        Some(MetadataValue::U64(value)) => *value,
        _ => 0,
    }
}

fn convert_snapshot(
    bus: &str,
    desktop_entry: Option<String>,
    source: &Metadata,
    status: PlaybackStatus,
    position_microseconds: u64,
    rate: f64,
) -> (MediaMetadata, PlaybackState) {
    let artists = source
        .artists()
        .filter(|artists| !artists.is_empty())
        .map(|artists| artists.join(", "));
    let metadata = MediaMetadata {
        bundle_identifier: Some(desktop_entry.unwrap_or_else(|| bus.to_owned())),
        title: source.title().map(str::to_owned),
        artist: artists,
        album: source.album_name().map(str::to_owned),
        duration: duration_microseconds(source) as f64 / 1_000_000.0,
        artwork_data: None,
        artwork_mime_type: None,
        content_item_identifier: Some(content_identity(bus, source)),
    };
    let state = PlaybackState {
        playing: status == PlaybackStatus::Playing,
        playback_rate: if rate.is_finite() && rate > 0.0 {
            rate
        } else {
            1.0
        },
        // A negative D-Bus Position is cast to u64 by mpris. Reject that
        // impossible wrapped value rather than publishing centuries of playback.
        elapsed_time: if position_microseconds <= i64::MAX as u64 {
            position_microseconds as f64 / 1_000_000.0
        } else {
            0.0
        },
    };
    (metadata, state)
}

fn content_identity(bus: &str, source: &Metadata) -> String {
    // An unambiguous, lossless tuple avoids delimiter/hash collisions and also
    // handles players that reuse a track path (or omit it entirely).
    serde_json::to_string(&(
        bus,
        source.get("mpris:trackid").and_then(MetadataValue::as_str),
        source.url(),
        source.title(),
        source.artists(),
        source.album_name(),
        duration_microseconds(source),
        source.art_url(),
    ))
    .expect("serializing a tuple of strings and integers cannot fail")
}

impl MediaContext {
    fn artwork_for(&mut self, key: (String, String)) -> Option<Artwork> {
        if let Some(worker) = &mut self.worker {
            if let Ok(result) = worker.results.try_recv() {
                worker.pending = false;
                self.artwork = Some(result);
            }
        }
        if let Some(cached) = &self.artwork {
            if cached.key == key
                && (cached.value.is_some() || cached.attempted.elapsed() < ARTWORK_RETRY)
            {
                return cached.value.clone();
            }
        }
        let url = url::Url::parse(&key.1).ok()?;
        if !matches!(url.scheme(), "file" | "http" | "https") {
            return None;
        }
        if self.worker.is_none() {
            self.worker = ArtworkWorker::start();
        }
        let worker = self.worker.as_mut()?;
        // At most one outstanding request and one worker. A blocked network
        // filesystem cannot hang polling or create a thread on every poll.
        if worker.pending {
            return None;
        }
        worker.requests.try_send(key.clone()).ok()?;
        worker.pending = true;
        let result = worker.results.recv_timeout(ARTWORK_TIMEOUT).ok();
        if let Some(result) = result {
            worker.pending = false;
            self.artwork = Some(result);
        } else {
            self.artwork = Some(CachedArtwork {
                key,
                value: None,
                attempted: Instant::now(),
            });
        }
        self.artwork.as_ref()?.value.clone()
    }
}

impl ArtworkWorker {
    fn start() -> Option<Self> {
        let (requests, incoming) = mpsc::sync_channel::<(String, String)>(1);
        let (outgoing, results) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("linux-media-artwork".into())
            .spawn(move || {
                let mut http = None;
                while let Ok(key) = incoming.recv() {
                    let value = load_artwork(&key.1, &mut http);
                    if outgoing
                        .send(CachedArtwork {
                            key,
                            value,
                            attempted: Instant::now(),
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .ok()?;
        Some(Self {
            requests,
            results,
            pending: false,
        })
    }
}

fn load_artwork(location: &str, http: &mut Option<reqwest::blocking::Client>) -> Option<Artwork> {
    let url = url::Url::parse(location).ok()?;
    match url.scheme() {
        "file" => read_file_artwork(&url),
        "http" | "https" => {
            if http.is_none() {
                *http = reqwest::blocking::Client::builder()
                    .connect_timeout(Duration::from_millis(800))
                    .timeout(ARTWORK_TIMEOUT)
                    .redirect(reqwest::redirect::Policy::limited(3))
                    .build()
                    .ok();
            }
            let response = http
                .as_ref()?
                .get(url)
                .send()
                .ok()?
                .error_for_status()
                .ok()?;
            if response
                .content_length()
                .is_some_and(|length| length > MAX_ARTWORK_BYTES)
            {
                return None;
            }
            artwork_from_bytes(read_bounded(response)?)
        }
        _ => None,
    }
}

fn read_bounded(reader: impl Read) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_ARTWORK_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= MAX_ARTWORK_BYTES).then_some(bytes)
}

fn read_file_artwork(url: &url::Url) -> Option<Artwork> {
    let path = url.to_file_path().ok()?;
    // Linux O_NONBLOCK prevents a replaced file/FIFO from blocking open().
    // Check the opened descriptor, not a race-prone path metadata lookup.
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(0x800)
        .open(path)
        .ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() > MAX_ARTWORK_BYTES {
        return None;
    }
    artwork_from_bytes(read_bounded(file)?)
}

fn artwork_from_bytes(bytes: Vec<u8>) -> Option<Artwork> {
    let format = image::guess_format(&bytes).ok()?;
    Some(Artwork {
        bytes,
        mime: format.to_mime_type().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mpris::MetadataValue;
    use std::collections::HashMap;
    use std::io::Write;

    fn track(values: &[(&str, MetadataValue)]) -> mpris::Metadata {
        mpris::Metadata::from(
            values
                .iter()
                .map(|(key, value)| (key.to_string(), value.clone()))
                .collect::<HashMap<_, _>>(),
        )
    }

    #[test]
    fn playing_preempts_paused_and_same_priority_selection_is_sticky() {
        let mut candidates = vec![
            ("z", PlaybackStatus::Playing),
            ("a", PlaybackStatus::Paused),
            ("b", PlaybackStatus::Playing),
        ];
        candidates.sort_by_key(|(bus, status)| selection_key(bus, *status, Some("z")));
        assert_eq!(candidates[0].0, "z");
        candidates.sort_by_key(|(bus, status)| selection_key(bus, *status, None));
        assert_eq!(candidates[0].0, "b");
        assert!(status_priority(PlaybackStatus::Stopped).is_none());
    }

    #[test]
    fn snapshot_preserves_all_artists_and_fractional_microseconds() {
        let source = track(&[
            ("xesam:title", MetadataValue::String("Track".into())),
            ("xesam:album", MetadataValue::String("Album".into())),
            (
                "xesam:artist",
                MetadataValue::Array(vec![
                    MetadataValue::String("First".into()),
                    MetadataValue::String("Second".into()),
                ]),
            ),
            ("mpris:length", MetadataValue::I64(12_345_678)),
        ]);
        let (metadata, state) = convert_snapshot(
            "org.mpris.MediaPlayer2.test",
            Some("test.desktop".into()),
            &source,
            PlaybackStatus::Paused,
            1_234_567,
            1.25,
        );
        assert_eq!(metadata.title.as_deref(), Some("Track"));
        assert_eq!(metadata.album.as_deref(), Some("Album"));
        assert_eq!(metadata.artist.as_deref(), Some("First, Second"));
        assert_eq!(metadata.bundle_identifier.as_deref(), Some("test.desktop"));
        assert!((metadata.duration - 12.345678).abs() < 1e-9);
        assert_eq!(
            state,
            PlaybackState {
                playing: false,
                playback_rate: 1.25,
                elapsed_time: 1.234567
            }
        );
    }

    #[test]
    fn absent_optional_metadata_still_returns_same_player_state() {
        let (metadata, state) = convert_snapshot(
            "org.mpris.MediaPlayer2.test",
            None,
            &mpris::Metadata::default(),
            PlaybackStatus::Playing,
            0,
            f64::NAN,
        );
        assert_eq!(metadata.title, None);
        assert_eq!(metadata.artist, None);
        assert_eq!(metadata.duration, 0.0);
        assert_eq!(
            metadata.bundle_identifier.as_deref(),
            Some("org.mpris.MediaPlayer2.test")
        );
        assert_eq!(
            state,
            PlaybackState {
                playing: true,
                playback_rate: 1.0,
                elapsed_time: 0.0
            }
        );
    }

    #[test]
    fn identity_separates_players_and_ambiguous_text_boundaries() {
        let first = track(&[
            ("xesam:title", MetadataValue::String("a:b".into())),
            ("xesam:album", MetadataValue::String("c".into())),
        ]);
        let second = track(&[
            ("xesam:title", MetadataValue::String("a".into())),
            ("xesam:album", MetadataValue::String("b:c".into())),
        ]);
        assert_ne!(
            content_identity("one", &first),
            content_identity("two", &first)
        );
        assert_ne!(
            content_identity("one", &first),
            content_identity("one", &second)
        );
        let fixed_id = track(&[
            ("mpris:trackid", MetadataValue::String("/track/1".into())),
            ("xesam:title", MetadataValue::String("Changed".into())),
        ]);
        assert_ne!(
            content_identity("one", &fixed_id),
            content_identity("one", &mpris::Metadata::new("/track/1"))
        );
    }

    #[test]
    fn negative_duration_is_not_reinterpreted_as_unsigned() {
        let source = track(&[("mpris:length", MetadataValue::I64(-1))]);
        let (metadata, state) = convert_snapshot(
            "test",
            None,
            &source,
            PlaybackStatus::Playing,
            u64::MAX,
            1.0,
        );
        assert_eq!(metadata.duration, 0.0);
        assert_eq!(state.elapsed_time, 0.0);
    }

    #[test]
    fn artwork_uses_bytes_not_file_extension_and_rejects_oversized_files() {
        let directory = std::env::temp_dir().join(format!(
            "shikenmatrix-art-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("misleading.jpg");
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(1, 1)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        std::fs::write(&path, png.get_ref()).unwrap();
        let artwork = read_file_artwork(&url::Url::from_file_path(&path).unwrap()).unwrap();
        assert_eq!(artwork.mime, "image/png");
        assert_eq!(artwork.bytes, *png.get_ref());
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(png.get_ref()).unwrap();
        file.set_len(MAX_ARTWORK_BYTES).unwrap();
        assert_eq!(
            read_file_artwork(&url::Url::from_file_path(&path).unwrap())
                .unwrap()
                .bytes
                .len() as u64,
            MAX_ARTWORK_BYTES
        );
        file.set_len(MAX_ARTWORK_BYTES + 1).unwrap();
        assert!(read_file_artwork(&url::Url::from_file_path(&path).unwrap()).is_none());
        file.set_len(0).unwrap();
        file.write_all(b"not an image").unwrap();
        assert!(read_file_artwork(&url::Url::from_file_path(&path).unwrap()).is_none());
        assert!(read_file_artwork(&url::Url::from_directory_path(&directory).unwrap()).is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn bounded_reader_rejects_oversize_even_without_a_declared_length() {
        assert!(read_bounded(std::io::repeat(0).take(MAX_ARTWORK_BYTES + 1)).is_none());
        assert_eq!(read_bounded(&b"abc"[..]), Some(b"abc".to_vec()));
    }

    #[test]
    fn current_artwork_is_cached_after_source_disappears() {
        let path =
            std::env::temp_dir().join(format!("shikenmatrix-cache-{}.png", std::process::id()));
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(1, 1)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        std::fs::write(&path, png.get_ref()).unwrap();
        let key = (
            "track".to_string(),
            url::Url::from_file_path(&path).unwrap().to_string(),
        );
        let mut context = MediaContext::default();
        let first = context.artwork_for(key.clone()).unwrap();
        std::fs::remove_file(path).unwrap();
        let second = context.artwork_for(key).unwrap();
        assert_eq!(first.bytes, second.bytes);
        assert_eq!(second.mime, "image/png");
    }
}
