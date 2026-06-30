use crate::reporter::{ReporterConfig, ReporterProtocol};
use crate::state::{ActivityEntry, ClientKind};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const UPSTREAM_SETTINGS_KEY: &str = "upstream_settings";
const RUNTIME_STATE_KEY: &str = "runtime_state";
const ACCESS_SETTINGS_KEY: &str = "access_settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessSettings {
    pub accept_desktop: bool,
    pub accept_mobile: bool,
    pub activity_log_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientKeyEntry {
    pub id: u64,
    pub description: String,
    pub api_key: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamSettings {
    pub protocol: ReporterProtocol,
    pub enable_media_reporting: bool,
    pub native_ws_url: String,
    pub native_token: String,
    pub mix_space_endpoint: String,
    pub mix_space_method: String,
    pub mix_space_token: String,
    pub s3_enabled: bool,
    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_endpoint: String,
    pub s3_custom_domain: String,
    pub s3_key_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedRuntimeState {
    #[serde(default)]
    pub total_messages: u64,
    #[serde(default)]
    pub window_info_count: u64,
    #[serde(default)]
    pub media_playback_count: u64,
    #[serde(default)]
    pub artwork_uploads: u64,
    #[serde(default)]
    pub upstream_errors: u64,
    #[serde(default)]
    pub last_activity_at: Option<u64>,
    #[serde(default)]
    pub current_window: Option<String>,
    #[serde(default)]
    pub current_media: Option<String>,
    #[serde(default)]
    pub desktop_messages: u64,
    #[serde(default)]
    pub mobile_messages: u64,
}

#[derive(Clone)]
pub struct Storage {
    connection: Arc<Mutex<Connection>>,
    database_path: PathBuf,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error> {
        let database_path = path.as_ref().to_path_buf();
        let connection = Connection::open(&database_path)?;
        connection.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS client_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT NOT NULL,
                api_key TEXT UNIQUE NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS activity_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts INTEGER NOT NULL,
                kind TEXT NOT NULL,
                client_kind TEXT,
                client_id INTEGER,
                summary TEXT NOT NULL,
                detail TEXT
            );
            "#,
        )?;

        Self::init_defaults(&connection)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            database_path,
        })
    }

    pub fn asset_dir(&self) -> PathBuf {
        self.database_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".cache")
    }

    fn init_defaults(connection: &Connection) -> Result<(), rusqlite::Error> {
        let mut statement =
            connection.prepare("SELECT value FROM app_settings WHERE key = 'jwt_secret'")?;
        let jwt_secret: Option<String> = statement.query_row([], |row| row.get(0)).optional()?;
        if jwt_secret.is_none() {
            let secret = generate_random_string(32);
            connection.execute(
                "INSERT INTO app_settings (key, value, updated_at) VALUES ('jwt_secret', ?1, unixepoch())",
                params![secret],
            )?;
        }

        let mut statement = connection.prepare("SELECT id FROM users WHERE username = 'admin'")?;
        let admin_exists: Option<i64> = statement.query_row([], |row| row.get(0)).optional()?;
        if admin_exists.is_none() {
            let password = generate_random_string(12);
            let hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap();
            connection.execute(
                "INSERT INTO users (username, password_hash, created_at) VALUES ('admin', ?1, unixepoch())",
                params![hash],
            )?;
            println!("=======================================================");
            println!("INITIAL ADMIN PASSWORD GENERATED:");
            println!("Username: admin");
            println!("Password: {}", password);
            println!("Please login and save this password.");
            println!("=======================================================");
        }
        Ok(())
    }

    pub fn load_upstream_settings(&self) -> UpstreamSettings {
        self.load_json_setting(UPSTREAM_SETTINGS_KEY)
            .unwrap_or_default()
    }

    pub fn save_upstream_settings(&self, settings: &UpstreamSettings) -> Result<(), String> {
        self.save_json_setting(UPSTREAM_SETTINGS_KEY, settings)
    }

    pub fn load_runtime_state(&self) -> PersistedRuntimeState {
        self.load_json_setting(RUNTIME_STATE_KEY)
            .unwrap_or_default()
    }

    pub fn save_runtime_state(&self, state: &PersistedRuntimeState) -> Result<(), String> {
        self.save_json_setting(RUNTIME_STATE_KEY, state)
    }

    pub fn load_access_settings(&self) -> AccessSettings {
        self.load_json_setting(ACCESS_SETTINGS_KEY)
            .unwrap_or_default()
    }

    pub fn save_access_settings(&self, settings: &AccessSettings) -> Result<(), String> {
        self.save_json_setting(ACCESS_SETTINGS_KEY, settings)
    }

    pub fn change_password(
        &self,
        username: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), String> {
        if !self.verify_user(username, current_password) {
            return Err("current password is incorrect".to_string());
        }
        if new_password.trim().is_empty() {
            return Err("new password must not be empty".to_string());
        }
        let hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
        let connection = self.connection.lock().map_err(|_| "lock error")?;
        connection
            .execute(
                "UPDATE users SET password_hash = ?1 WHERE username = ?2",
                params![hash, username],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_activity(&self) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "lock error")?;
        connection
            .execute("DELETE FROM activity_events", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn count_activity(&self) -> u64 {
        let Ok(connection) = self.connection.lock() else {
            return 0;
        };
        connection
            .query_row("SELECT COUNT(*) FROM activity_events", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|n| n as u64)
            .unwrap_or(0)
    }

    pub fn load_recent_activity(&self, limit: usize) -> Vec<ActivityEntry> {
        let Ok(connection) = self.connection.lock() else {
            return Vec::new();
        };
        let Ok(mut statement) = connection.prepare(
            r#"
            SELECT ts, kind, client_kind, client_id, summary, detail
            FROM activity_events
            ORDER BY id DESC
            LIMIT ?1
            "#,
        ) else {
            return Vec::new();
        };

        let Ok(rows) = statement.query_map([limit as i64], |row| {
            let kind: String = row.get(1)?;
            let client_kind: Option<String> = row.get(2)?;
            let client_id: Option<i64> = row.get(3)?;
            Ok(ActivityEntry {
                ts: row.get::<_, i64>(0)? as u64,
                kind: activity_kind_label(&kind),
                client: client_kind.as_deref().and_then(parse_client_kind),
                client_id: client_id.map(|id| id as u64),
                summary: row.get(4)?,
                detail: row.get(5)?,
            })
        }) else {
            return Vec::new();
        };

        let mut activity = rows.filter_map(Result::ok).collect::<Vec<_>>();
        activity.reverse();
        activity
    }

    pub fn get_jwt_secret(&self) -> String {
        self.load_json_setting::<String>("jwt_secret")
            .unwrap_or_else(|| {
                // If it fails to parse as JSON (because it's a raw string in DB), query it directly
                let Ok(connection) = self.connection.lock() else {
                    return "".to_string();
                };
                connection
                    .query_row(
                        "SELECT value FROM app_settings WHERE key = 'jwt_secret'",
                        [],
                        |row| row.get::<_, String>(0),
                    )
                    .unwrap_or_default()
            })
    }

    pub fn verify_user(&self, username: &str, password: &str) -> bool {
        let Ok(connection) = self.connection.lock() else {
            return false;
        };
        let Ok(hash) = connection.query_row(
            "SELECT password_hash FROM users WHERE username = ?1",
            params![username],
            |row| row.get::<_, String>(0),
        ) else {
            return false;
        };
        bcrypt::verify(password, &hash).unwrap_or(false)
    }

    pub fn create_client_key(&self, description: &str) -> Result<String, String> {
        let api_key = format!("sk_{}", generate_random_string(32));
        let connection = self.connection.lock().map_err(|_| "lock error")?;
        connection.execute(
            "INSERT INTO client_keys (description, api_key, created_at) VALUES (?1, ?2, unixepoch())",
            params![description, api_key],
        ).map_err(|e| e.to_string())?;
        Ok(api_key)
    }

    pub fn get_client_keys(&self) -> Vec<ClientKeyEntry> {
        let Ok(connection) = self.connection.lock() else {
            return Vec::new();
        };
        let Ok(mut stmt) = connection.prepare(
            "SELECT id, description, api_key, created_at FROM client_keys ORDER BY id DESC",
        ) else {
            return Vec::new();
        };
        let rows = stmt.query_map([], |row| {
            Ok(ClientKeyEntry {
                id: row.get::<_, i64>(0)? as u64,
                description: row.get(1)?,
                api_key: row.get(2)?,
                created_at: row.get::<_, i64>(3)? as u64,
            })
        });
        rows.map(|r| r.filter_map(Result::ok).collect())
            .unwrap_or_default()
    }

    pub fn delete_client_key(&self, id: u64) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "lock error")?;
        connection
            .execute("DELETE FROM client_keys WHERE id = ?1", params![id as i64])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn verify_client_key(&self, api_key: &str) -> bool {
        let Ok(connection) = self.connection.lock() else {
            return false;
        };
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM client_keys WHERE api_key = ?1",
                params![api_key],
                |row| row.get(0),
            )
            .unwrap_or(0);
        count > 0
    }

    fn load_json_setting<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.connection
            .lock()
            .ok()
            .and_then(|connection| {
                connection
                    .query_row(
                        "SELECT value FROM app_settings WHERE key = ?1",
                        [key],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .ok()
                    .flatten()
            })
            .and_then(|value| serde_json::from_str::<T>(&value).ok())
    }

    fn save_json_setting<T>(&self, key: &str, settings: &T) -> Result<(), String>
    where
        T: Serialize,
    {
        let value = serde_json::to_string(settings).map_err(|error| error.to_string())?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "storage lock poisoned".to_string())?;
        connection
            .execute(
                r#"
                INSERT INTO app_settings (key, value, updated_at)
                VALUES (?1, ?2, unixepoch())
                ON CONFLICT(key) DO UPDATE SET
                  value = excluded.value,
                  updated_at = excluded.updated_at
                "#,
                params![key, value],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn record_activity(&self, entry: &ActivityEntry) {
        let Ok(connection) = self.connection.lock() else {
            return;
        };
        let _ = connection.execute(
            r#"
            INSERT INTO activity_events (ts, kind, client_kind, client_id, summary, detail)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                entry.ts as i64,
                entry.kind,
                entry.client.map(client_kind_label),
                entry.client_id.map(|id| id as i64),
                entry.summary,
                entry.detail,
            ],
        );
    }
}

impl UpstreamSettings {
    pub fn to_reporter_config(&self) -> Option<ReporterConfig> {
        let mut config = ReporterConfig {
            protocol: self.protocol,
            enable_media_reporting: self.enable_media_reporting,
            ..ReporterConfig::default()
        };

        config.native.ws_url = self.native_ws_url.clone();
        config.native.token = self.native_token.clone();
        config.mix_space.endpoint = self.mix_space_endpoint.clone();
        config.mix_space.method = self.mix_space_method.clone();
        config.mix_space.token = self.mix_space_token.clone();
        config.s3.enabled = self.s3_enabled;
        config.s3.bucket = self.s3_bucket.clone();
        config.s3.region = self.s3_region.clone();
        config.s3.access_key = self.s3_access_key.clone();
        config.s3.secret_key = self.s3_secret_key.clone();
        config.s3.endpoint = self.s3_endpoint.clone();
        config.s3.custom_domain = self.s3_custom_domain.clone();
        config.s3.key_template = self.s3_key_template.clone();

        match config.protocol {
            ReporterProtocol::Native
                if config.native.ws_url.trim().is_empty()
                    || config.native.token.trim().is_empty() =>
            {
                None
            }
            ReporterProtocol::MixSpace
                if config.mix_space.endpoint.trim().is_empty()
                    || config.mix_space.token.trim().is_empty() =>
            {
                None
            }
            _ => Some(config),
        }
    }
}

impl Default for UpstreamSettings {
    fn default() -> Self {
        Self {
            protocol: ReporterProtocol::Native,
            enable_media_reporting: true,
            native_ws_url: String::new(),
            native_token: String::new(),
            mix_space_endpoint: String::new(),
            mix_space_method: "POST".to_string(),
            mix_space_token: String::new(),
            s3_enabled: false,
            s3_bucket: String::new(),
            s3_region: "us-east-1".to_string(),
            s3_access_key: String::new(),
            s3_secret_key: String::new(),
            s3_endpoint: String::new(),
            s3_custom_domain: String::new(),
            s3_key_template: "{kind}/{Y}/{M}/{D}/{SHA}.{ext}".to_string(),
        }
    }
}

impl Default for AccessSettings {
    fn default() -> Self {
        Self {
            accept_desktop: true,
            accept_mobile: true,
            activity_log_limit: 120,
        }
    }
}

fn client_kind_label(kind: ClientKind) -> &'static str {
    match kind {
        ClientKind::DesktopReporter => "desktop_reporter",
        ClientKind::Mobile => "mobile",
    }
}

fn parse_client_kind(kind: &str) -> Option<ClientKind> {
    match kind {
        "desktop_reporter" => Some(ClientKind::DesktopReporter),
        "mobile" => Some(ClientKind::Mobile),
        _ => None,
    }
}

fn activity_kind_label(kind: &str) -> &'static str {
    match kind {
        "window_info" => "window_info",
        "media_playback" => "media_playback",
        "artwork_upload" => "artwork_upload",
        "client_connect" => "client_connect",
        "client_disconnect" => "client_disconnect",
        "client_rejected" => "client_rejected",
        "config_update" => "config_update",
        "upstream_error" => "upstream_error",
        _ => "unknown",
    }
}

fn generate_random_string(length: usize) -> String {
    let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..length)
        .map(|_| {
            let idx = rand::random::<u8>() % 62;
            chars[idx as usize] as char
        })
        .collect()
}
