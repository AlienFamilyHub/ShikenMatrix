use serde::{Deserialize, Serialize};
use std::fs;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
pub struct S3Config {
    pub s3_enable: bool,
    pub upload_path: String,
    pub endpoint: String,
    pub region: String,
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub custom_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub endpoint: String,
    pub token: String,
    pub report_time: i32,
    pub report_smtc: bool,
    pub skip_smtc_cover: bool,
    pub upload_smtc_cover: bool,
    pub log_base64: bool,
    // S3配置
    pub s3_config: S3Config,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub match_application: String,
    pub replace: Replace,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Replace {
    pub application: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MainConfig {
    pub server_config: ServerConfig,
    pub rules: Vec<Rule>,
}

pub fn load_config() -> MainConfig {
    let workdir = std::env::current_dir().unwrap();
    let config_path = if cfg!(dev) {
        workdir.join("../config.yml")
    } else {
        workdir.join("config.yml")
    };
    let data = fs::read_to_string(&config_path).unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        exit(1);
    });
    let config: MainConfig = serde_yaml::from_str(&data).unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        exit(1);
    });
    return config;
}
