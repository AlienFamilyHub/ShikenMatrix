use chrono::Local;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct Cache {
    uploads: HashMap<String, String>,
}

fn load_cache(cache_path: &PathBuf) -> Cache {
    if let Ok(mut file) = File::open(cache_path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(cache) = serde_json::from_str(&contents) {
                return cache;
            }
        }
    }
    Cache {
        uploads: HashMap::new(),
    }
}

fn save_cache(cache: &Cache, cache_path: &PathBuf) {
    if let Ok(contents) = serde_json::to_string_pretty(cache) {
        let _ = fs::write(cache_path, contents);
    }
}

fn replace_bucket_path_template(template: &str) -> String {
    let now = Local::now();
    template
        .replace("{year}", &now.format("%Y").to_string())
        .replace("{month}", &now.format("%m").to_string())
        .replace("{day}", &now.format("%d").to_string())
}

// make by chatgpt
pub async fn upload_images(image_data: &[u8]) -> Result<String, String> {
    // 计算文件 hash
    let hash = format!("{:x}", Sha256::digest(image_data));
    let filename = format!("{}.webp", hash);

    // 构建路径前缀（基于配置中的 upload_path）
    let config = crate::modules::get_config::load_config();
    let s3_config = config.server_config.s3_config;

    if !s3_config.s3_enable {
        return Err("S3 未启用".into());
    }

    let prefix = replace_bucket_path_template(&s3_config.upload_path);
    let full_path = if prefix.is_empty() {
        filename.clone()
    } else {
        format!("{}/{}", prefix.trim_matches('/'), filename)
    };

    // 检查本地缓存是否存在该文件上传记录
    let cache_dir = crate::libs::cache::get_cache_directory();
    let cache_path = cache_dir.join("uploads.json");
    let mut cache = load_cache(&cache_path);

    if let Some(cached_url) = cache.uploads.get(&hash) {
        return Ok(cached_url.clone());
    }

    // 初始化 S3 储存桶 Bucket（在上传前完成！）
    let region = Region::Custom {
        region: s3_config.region.clone(),
        endpoint: s3_config.endpoint.clone(),
    };

    let credentials = Credentials::new(
        Some(&s3_config.access_key),
        Some(&s3_config.secret_key),
        None,
        None,
        None,
    )
    .map_err(|e| format!("S3 凭证创建失败: {}", e))?;

    let bucket = Bucket::new(&s3_config.bucket_name, region, credentials)
        .map_err(|e| format!("S3 Bucket 初始化失败: {}", e))?
        .with_path_style(); // important for minio or custom endpoints

    // 转换图像为 WebP 格式
    let webp_data = crate::modules::files_converter::convert_to_webp(image_data)?;

    // 执行上传操作
    bucket
        .put_object_with_content_type(&full_path, &webp_data, "image/webp")
        .await
        .map_err(|e| format!("上传失败: {}", e))?;

    // 构造URL
    let url = if !s3_config.custom_url.is_empty() {
        format!(
            "{}/{}",
            s3_config.custom_url.trim_end_matches('/'),
            full_path
        )
    } else {
        format!(
            "{}/{}/{}",
            s3_config.endpoint.trim_end_matches('/'),
            s3_config.bucket_name,
            full_path
        )
    };

    // 写入缓存并返回
    cache.uploads.insert(hash, url.clone());
    save_cache(&cache, &cache_path);

    Ok(url)
}
