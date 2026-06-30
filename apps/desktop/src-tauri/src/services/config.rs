//! Configuration file management
//! Persists configuration to config.toml

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

use super::ReporterConfig;

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseBehavior {
    HideToTray,
    Quit,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub reporter: ReporterConfig,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub close_behavior: Option<CloseBehavior>,
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Get config file path (config.toml in user data directory)
fn get_config_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        let config_dir = home.join(".shikenmatrix");
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
            info!("Created config directory: {}", config_dir.display());
        }
        let path = config_dir.join(CONFIG_FILE);
        info!("Config path: {}", path.display());
        return path;
    }

    let path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(CONFIG_FILE);
    info!("Config path (fallback): {}", path.display());
    path
}

/// Load configuration
pub fn load_config() -> AppConfig {
    let path = get_config_path();

    if !path.exists() {
        info!("Config file not found, using defaults");
        let mut config = AppConfig::default();
        normalize_config(&mut config);
        let _ = write_config_to_path(&path, &config);
        return config;
    }

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(mut config) => {
                migrate_flat_reporter_config(&mut config, &content);
                if normalize_config(&mut config) {
                    let _ = write_config_to_path(&path, &config);
                }
                info!("Config loaded successfully: {}", path.display());
                config
            }
            Err(e) => {
                info!("Failed to parse config: {}, using defaults", e);
                AppConfig::default()
            }
        },
        Err(e) => {
            info!("Failed to read config: {}, using defaults", e);
            AppConfig::default()
        }
    }
}

fn migrate_flat_reporter_config(config: &mut AppConfig, content: &str) {
    let Ok(value) = content.parse::<toml::Value>() else {
        return;
    };
    let Some(reporter) = value.get("reporter") else {
        return;
    };

    fill_string_if_empty(
        &mut config.reporter.server.ws_url,
        reporter,
        "server_ws_url",
    );
    fill_string_if_empty(
        &mut config.reporter.server.api_key,
        reporter,
        "server_api_key",
    );
    fill_string_if_empty(&mut config.reporter.server.api_key, reporter, "api_key");
    fill_string_if_empty(
        &mut config.reporter.server.client,
        reporter,
        "client",
    );
    fill_string_if_empty(
        &mut config.reporter.server.device_id,
        reporter,
        "device_id",
    );
    fill_string_if_empty(
        &mut config.reporter.server.device_id,
        reporter,
        "deviceId",
    );
}

fn fill_string_if_empty(target: &mut String, source: &toml::Value, key: &str) {
    if !target.is_empty() {
        return;
    }

    if let Some(value) = source.get(key).and_then(toml::Value::as_str) {
        *target = value.to_string();
    }
}

fn normalize_config(config: &mut AppConfig) -> bool {
    let mut changed = false;

    if config.reporter.server.client.is_empty() {
        config.reporter.server.client = default_desktop_client_info();
        changed = true;
    }

    if config.reporter.server.device_id.is_empty() {
        config.reporter.server.device_id = generate_desktop_device_id();
        changed = true;
    }

    changed
}

fn default_desktop_client_info() -> String {
    format!("desktop-{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn generate_desktop_device_id() -> String {
    let mut hasher = DefaultHasher::new();
    std::env::consts::OS.hash(&mut hasher);
    std::env::consts::ARCH.hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    std::env::var("USER").ok().hash(&mut hasher);
    std::env::var("USERNAME").ok().hash(&mut hasher);
    std::env::current_exe().ok().hash(&mut hasher);
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_nanos())
        .hash(&mut hasher);

    format!("desktop_{:016x}", hasher.finish())
}

/// Save configuration
#[allow(dead_code)]
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();

    write_config_to_path(&path, config)
}

fn write_config_to_path(path: &PathBuf, config: &AppConfig) -> Result<(), String> {
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    info!("Writing config to: {}", path.display());

    fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

    info!("Config saved successfully to: {}", path.display());
    Ok(())
}

/// Update reporter configuration and save
pub fn save_reporter_config(reporter_config: &ReporterConfig) -> Result<(), String> {
    let mut config = load_config();
    config.reporter = reporter_config.clone();
    save_config(&config)
}

pub fn save_close_behavior(close_behavior: CloseBehavior) -> Result<(), String> {
    let mut config = load_config();
    config.close_behavior = Some(close_behavior);
    save_config(&config)
}
