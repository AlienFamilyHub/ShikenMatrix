//! Configuration file management
//! Persists configuration to config.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

use super::ReporterConfig;

const CONFIG_FILE: &str = "config.toml";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub reporter: ReporterConfig,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for ReporterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ws_url: String::new(),
            token: String::new(),
            enable_media_reporting: false,
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
        Ok(content) => {
            match toml::from_str(&content) {
                Ok(config) => {
                    info!("Config loaded successfully: {}", path.display());
                    config
                }
                Err(e) => {
                    info!("Failed to parse config: {}, using defaults", e);
                    AppConfig::default()
                }
            }
        }
        Err(e) => {
            info!("Failed to read config: {}, using defaults", e);
            AppConfig::default()
        }
    }
}

/// Save configuration
#[allow(dead_code)]
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();

    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    info!("Writing config to: {}", path.display());
    info!("Config content:\n{}", content);

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    info!("Config saved successfully to: {}", path.display());
    Ok(())
}

/// Update reporter configuration and save
pub fn save_reporter_config(reporter_config: &ReporterConfig) -> Result<(), String> {
    let mut config = load_config();
    config.reporter = reporter_config.clone();
    save_config(&config)
}

/// Get log level from config
pub fn get_log_level() -> String {
    let config = load_config();
    config.log_level
}
