use crate::modules::files_converter::encode_as_base64;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use tokio::runtime::Runtime;

// 定义常量
const HIDDEN_CONTENT: &str = "[BASE64_CONTENT_HIDDEN]";

pub fn report(
    endpoint: &str,
    token: &str,
) -> (
    String,
    HashMap<String, Value>,
    String,
    HashMap<String, Value>,
    String,
) {
    // 获取配置
    let config = crate::modules::get_config::load_config();

    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    // 获取前台进程信息
    let (process_name_raw, window_name, icon_bytes) =
        crate::modules::get_processes::get_window_info();

    let process_name =
        crate::modules::get_processes::replacer(&process_name_raw.replace(".exe", ""));

    // 获取媒体信息
    let (title, artist, source_app_name, thumbnail_bytes, duration, elapsed_time) =
        crate::modules::get_media::get_media_info();

    let thumbnail_base64 = encode_as_base64(&thumbnail_bytes);
    let icon_base64 = encode_as_base64(&icon_bytes);

    // 处理 thumbnail
    let thumbnail_to_use = if config.server_config.skip_smtc_cover {
        String::new()
    } else {
        if config.server_config.upload_smtc_cover && config.server_config.s3_config.s3_enable {
            match rt.block_on(crate::modules::upload_images::upload_images(
                &thumbnail_bytes,
            )) {
                Ok(url) if !url.is_empty() => url,
                _ => {
                    log::warn!("S3上传失败或返回为空，使用base64");
                    thumbnail_base64.clone()
                }
            }
        } else {
            thumbnail_base64.clone()
        }
    };

    // 处理 icon_bytes
    let icon_to_use = {
        if config.server_config.s3_config.s3_enable {
            match rt.block_on(crate::modules::upload_images::upload_images(&icon_bytes)) {
                Ok(url) if !url.is_empty() => url,
                _ => {
                    log::warn!("Icon上传失败或返回为空，使用base64");
                    icon_base64.clone()
                }
            }
        } else {
            icon_base64.clone()
        }
    };

    // 构建媒体更新
    let media_update = crate::modules::requests::build_media_update(
        &title,
        &artist,
        &source_app_name,
        &thumbnail_to_use,
        duration,
        elapsed_time,
    );

    let mut update_data = crate::modules::requests::build_data(
        &process_name,
        media_update.clone(),
        token,
        &icon_to_use,
    );

    let response = crate::modules::requests::send_request(update_data.clone(), endpoint);

    let status = !config.server_config.log_base64
        && config.server_config.report_smtc
        && !config.server_config.skip_smtc_cover
        && !config.server_config.upload_smtc_cover;

    let log_message = if status {
        if let Ok(mut json) = serde_json::from_str::<Value>(&response) {
            if let Some(media) = json.get_mut("media") {
                if let Some(thumbnail) = media.get_mut("AlbumThumbnail") {
                    if let Value::String(thumb_str) = thumbnail {
                        if thumb_str.contains("base64") {
                            *thumbnail = Value::String(HIDDEN_CONTENT.to_string());
                        }
                    }
                }
            }
            json.to_string()
        } else {
            lazy_static! {
                static ref BASE64_RE: Regex = Regex::new(
                    r#"(?i)("AlbumThumbnail":\s*")([^"]*base64[^"]*)(")|(data:image/[^;]+;base64,[^"]+)"#
                ).expect("Invalid regex pattern");
            }

            BASE64_RE
                .replace_all(&response, |caps: &regex::Captures| {
                    if let Some(_) = caps.get(2) {
                        format!("{}{}{}", &caps[1], HIDDEN_CONTENT, &caps[3])
                    } else {
                        format!("data:image/type;base64,{}", HIDDEN_CONTENT)
                    }
                })
                .to_string()
        }
    } else {
        response.to_string()
    };

    log::info!("{}", log_message);

    update_data.remove("key");
    update_data.insert(
        "window_name".to_string(),
        serde_json::Value::String(window_name.trim_end_matches('\u{0000}').to_string()),
    );

    (
        response,
        update_data,
        icon_base64,
        media_update,
        thumbnail_base64,
    )
}
