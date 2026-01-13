use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{to_value, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn build_media_update(
    title: &str,
    artist: &str,
    source_app_name: &str,
    thumbnail: &str,
    duration: i64,
    elapsed_time: i64,
) -> HashMap<String, Value> {
    let mut media_update = HashMap::new();
    media_update.insert("title".to_string(), Value::String(title.to_string()));
    media_update.insert("artist".to_string(), Value::String(artist.to_string()));
    
    // 获取配置并应用进程名称替换规则
    let config = crate::modules::get_config::load_config();
    let process_name = config.rules.iter()
        .find(|rule| rule.match_application == source_app_name)
        .map(|rule| rule.replace.application.as_str())
        .unwrap_or(source_app_name);
    
    media_update.insert(
        "processName".to_string(),
        Value::String(process_name.to_string()),
    );
    media_update.insert(
        "AlbumThumbnail".to_string(),
        Value::String(thumbnail.to_string()),
    );
    media_update.insert("duration".to_string(), Value::Number(duration.into()));
    media_update.insert(
        "elapsedTime".to_string(),
        Value::Number(elapsed_time.into()),
    );
    media_update
}

pub fn build_data(
    process_name: &str,
    media_update: HashMap<String, Value>,
    token: &str,
    icon: &str,
) -> HashMap<String, Value> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs() as i64;

    let mut update_data = HashMap::new();

    update_data.insert("timestamp".to_string(), Value::from(timestamp));


    update_data.insert("key".to_string(), Value::from(token));

    // 新增 processInfo 字段
    let mut process_info = serde_json::Map::new();
    process_info.insert(
        "name".to_string(),
        Value::from(process_name.trim_end_matches('\0')),
    );
    process_info.insert(
        "description".to_string(),
        Value::from(process_name.trim_end_matches('\0')),
    );
    if icon.starts_with("http://") || icon.starts_with("https://") {
        process_info.insert("iconUrl".to_string(), Value::from(icon));
    } else if !icon.is_empty() {
        process_info.insert("iconBase64".to_string(), Value::from(icon));
    }

    update_data.insert(
        "process".to_string(),
        to_value(process_info).expect("Failed to convert process_info to Value"),
    );

    if let Some(title) = media_update.get("title") {
        if title.is_string()
            && !title
                .as_str()
                .expect("Failed to convert title to Value")
                .is_empty()
        {
            update_data.insert(
                "media".to_string(),
                to_value(media_update).expect("Failed to convert media_update to Value"),
            );
        }
    }

    update_data
}

pub fn send_request(update_data: HashMap<String, Value>, endpoint: &str) -> String {
    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );

    headers.insert(
        HeaderName::from_static("user-agent"),
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; TokaiTeio) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.1 Safari/537.36 Edg/114.0.1823.82 iykrzu/114.514"),
    );

    let response = client
        .post(endpoint)
        .headers(headers)
        .json(&update_data)
        .send();

    match response {
        Ok(resp) => match resp.text() {
            Ok(text) => text,
            Err(err) => format!("读取响应体失败: {}", err),
        },
        Err(err) => format!("请求失败: {}", err),
    }
}
