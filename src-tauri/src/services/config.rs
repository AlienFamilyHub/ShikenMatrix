//! 配置文件管理
//! 将配置持久化到 config.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::ReporterConfig;

const CONFIG_FILE: &str = "config.toml";

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub reporter: ReporterConfig,
}

impl Default for ReporterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ws_url: String::new(),
            token: String::new(),
        }
    }
}

/// 获取配置文件路径（用户数据目录下的 config.toml）
fn get_config_path() -> PathBuf {
    // 优先使用用户主目录下的 .shikenmatrix/config.toml
    if let Some(home) = dirs::home_dir() {
        let config_dir = home.join(".shikenmatrix");
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        return config_dir.join(CONFIG_FILE);
    }
    
    // 回退到当前目录
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(CONFIG_FILE)
}

/// 加载配置
pub fn load_config() -> AppConfig {
    let path = get_config_path();
    
    if !path.exists() {
        println!("📄 配置文件不存在，使用默认配置");
        return AppConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => {
            match toml::from_str(&content) {
                Ok(config) => {
                    println!("✅ 配置文件加载成功: {}", path.display());
                    config
                }
                Err(e) => {
                    eprintln!("❌ 配置文件解析失败: {}", e);
                    AppConfig::default()
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 配置文件读取失败: {}", e);
            AppConfig::default()
        }
    }
}

/// 保存配置
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    println!("✅ 配置已保存到: {}", path.display());
    Ok(())
}

/// 更新 reporter 配置并保存
pub fn save_reporter_config(reporter_config: &ReporterConfig) -> Result<(), String> {
    let mut config = load_config();
    config.reporter = reporter_config.clone();
    save_config(&config)
}
