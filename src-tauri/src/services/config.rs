//! Configuration file management
//! Persists configuration to config.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
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

impl Default for ReporterConfig {
    fn default() -> Self {
        Self {
            protocol: Default::default(),
            enable_media_reporting: false,
            native: Default::default(),
            mix_space: Default::default(),
            s3: Default::default(),
        }
    }
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
        return AppConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(mut config) => {
                migrate_flat_reporter_config(&mut config, &content);
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

    fill_string_if_empty(&mut config.reporter.native.ws_url, reporter, "ws_url");
    fill_string_if_empty(&mut config.reporter.native.token, reporter, "token");
    fill_string_if_empty(
        &mut config.reporter.mix_space.endpoint,
        reporter,
        "mix_space_endpoint",
    );
    fill_string_if_empty(
        &mut config.reporter.mix_space.method,
        reporter,
        "mix_space_method",
    );
    fill_string_if_empty(
        &mut config.reporter.mix_space.token,
        reporter,
        "mix_space_token",
    );
    fill_string_if_empty(&mut config.reporter.s3.bucket, reporter, "s3_bucket");
    fill_string_if_empty(&mut config.reporter.s3.region, reporter, "s3_region");
    fill_string_if_empty(
        &mut config.reporter.s3.access_key,
        reporter,
        "s3_access_key",
    );
    fill_string_if_empty(
        &mut config.reporter.s3.secret_key,
        reporter,
        "s3_secret_key",
    );
    fill_string_if_empty(&mut config.reporter.s3.endpoint, reporter, "s3_endpoint");
    fill_string_if_empty(
        &mut config.reporter.s3.custom_domain,
        reporter,
        "s3_custom_domain",
    );
    fill_string_if_empty(
        &mut config.reporter.s3.key_template,
        reporter,
        "s3_key_template",
    );

    if let Some(enabled) = reporter.get("s3_enabled").and_then(toml::Value::as_bool) {
        config.reporter.s3.enabled = enabled;
    }
    if let Some(days) = reporter
        .get("s3_lifecycle_days")
        .and_then(toml::Value::as_integer)
        .and_then(|value| u32::try_from(value).ok())
    {
        config.reporter.s3.lifecycle_days = days;
    }
}

fn fill_string_if_empty(target: &mut String, source: &toml::Value, key: &str) {
    if !target.is_empty() {
        return;
    }

    if let Some(value) = source.get(key).and_then(toml::Value::as_str) {
        *target = value.to_string();
    }
}

/// Save configuration
#[allow(dead_code)]
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();

    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    info!("Writing config to: {}", path.display());
    info!("Config content:\n{}", content);

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
