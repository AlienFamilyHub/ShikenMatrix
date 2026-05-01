use super::types::ReporterConfig;
use hmac::digest::KeyInit;
use hmac::{Hmac, Mac};
use reqwest::Method;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime};

type HmacSha256 = Hmac<Sha256>;

pub(super) async fn upload_app_icon(
    client: &reqwest::Client,
    config: &ReporterConfig,
    icon_data: &[u8],
    app_name: &str,
) -> Result<String, String> {
    upload_object(
        client,
        config,
        icon_data,
        "image/png",
        "png",
        "app-icons",
        app_name,
    )
    .await
}

pub(super) async fn upload_media_icon(
    client: &reqwest::Client,
    config: &ReporterConfig,
    icon_data: &[u8],
    mime_type: &str,
) -> Result<String, String> {
    let extension = extension_from_mime(mime_type);
    upload_object(
        client,
        config,
        icon_data,
        mime_type,
        extension,
        "media-icons",
        "media",
    )
    .await
}

async fn upload_object(
    client: &reqwest::Client,
    config: &ReporterConfig,
    data: &[u8],
    content_type: &str,
    extension: &str,
    kind: &str,
    label: &str,
) -> Result<String, String> {
    validate_s3_config(config)?;

    let object_key = object_key(config, data, extension, kind, label);
    if object_exists(client, config, &object_key).await? {
        return Ok(public_url(config, &object_key));
    }

    upload_to_s3(client, config, &object_key, data, content_type).await?;
    Ok(public_url(config, &object_key))
}

fn validate_s3_config(config: &ReporterConfig) -> Result<(), String> {
    if config.s3.bucket.trim().is_empty()
        || config.s3.region.trim().is_empty()
        || config.s3.access_key.trim().is_empty()
        || config.s3.secret_key.trim().is_empty()
    {
        return Err("S3 config is incomplete".to_string());
    }

    Ok(())
}

async fn object_exists(
    client: &reqwest::Client,
    config: &ReporterConfig,
    object_key: &str,
) -> Result<bool, String> {
    let response = signed_request(client, Method::HEAD, config, object_key, &[], None)
        .await?
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if response.status().is_success() {
        return Ok(true);
    }

    if response.status().as_u16() == 404 {
        return Ok(false);
    }

    Err(format!("S3 HEAD failed: {}", response.status()))
}

async fn upload_to_s3(
    client: &reqwest::Client,
    config: &ReporterConfig,
    object_key: &str,
    data: &[u8],
    content_type: &str,
) -> Result<(), String> {
    signed_request(
        client,
        Method::PUT,
        config,
        object_key,
        data,
        Some(content_type),
    )
    .await?
    .send()
    .await
    .map_err(|error| error.to_string())?
    .error_for_status()
    .map_err(|error| error.to_string())?;

    Ok(())
}

async fn signed_request(
    client: &reqwest::Client,
    method: Method,
    config: &ReporterConfig,
    object_key: &str,
    data: &[u8],
    content_type: Option<&str>,
) -> Result<reqwest::RequestBuilder, String> {
    let endpoint = endpoint(config);
    let host = reqwest::Url::parse(&endpoint)
        .map_err(|error| format!("Invalid S3 endpoint: {error}"))?
        .host_str()
        .ok_or_else(|| "S3 endpoint has no host".to_string())?
        .to_string();

    let amz_date = amz_date();
    let date_stamp = amz_date[..8].to_string();
    let payload_hash = sha256_hex(data);
    let canonical_uri = format!("/{}/{}", config.s3.bucket.trim(), object_key);
    let mut signed_values = BTreeMap::new();
    signed_values.insert("host".to_string(), host);
    signed_values.insert("x-amz-content-sha256".to_string(), payload_hash.clone());
    signed_values.insert("x-amz-date".to_string(), amz_date.clone());

    if let Some(content_type) = content_type {
        signed_values.insert("content-type".to_string(), content_type.to_string());
        add_lifecycle_headers(config, &mut signed_values);
    }

    let canonical_headers = signed_values
        .iter()
        .map(|(key, value)| format!("{key}:{}", value.trim()))
        .collect::<Vec<_>>()
        .join("\n");
    let signed_headers = signed_values.keys().cloned().collect::<Vec<_>>().join(";");
    let canonical_request = format!(
        "{}\n{canonical_uri}\n\n{canonical_headers}\n\n{signed_headers}\n{payload_hash}",
        method.as_str()
    );
    let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, config.s3.region.trim());
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );
    let signature = signature(config, &date_stamp, &string_to_sign)?;
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
        config.s3.access_key.trim()
    );

    let mut headers = HeaderMap::new();
    for (key, value) in signed_values {
        if key == "host" {
            continue;
        }
        let header_name =
            HeaderName::from_bytes(key.as_bytes()).map_err(|error| error.to_string())?;
        let header_value = HeaderValue::from_str(&value).map_err(|error| error.to_string())?;
        headers.insert(header_name, header_value);
    }
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&authorization).map_err(|error| error.to_string())?,
    );

    let url = format!("{endpoint}{canonical_uri}");
    let builder = client.request(method, url).headers(headers);
    if data.is_empty() {
        Ok(builder)
    } else {
        Ok(builder.body(data.to_vec()))
    }
}

fn add_lifecycle_headers(config: &ReporterConfig, headers: &mut BTreeMap<String, String>) {
    if config.s3.lifecycle_days == 0 {
        return;
    }

    let max_age = u64::from(config.s3.lifecycle_days) * 24 * 60 * 60;
    let expires = OffsetDateTime::now_utc() + Duration::days(i64::from(config.s3.lifecycle_days));
    headers.insert(
        "cache-control".to_string(),
        format!("public, max-age={max_age}"),
    );
    headers.insert("expires".to_string(), http_date(expires));
}

fn signature(
    config: &ReporterConfig,
    date_stamp: &str,
    string_to_sign: &str,
) -> Result<String, String> {
    let date_key = hmac_sha256(
        format!("AWS4{}", config.s3.secret_key.trim()).as_bytes(),
        date_stamp.as_bytes(),
    )?;
    let region_key = hmac_sha256(&date_key, config.s3.region.trim().as_bytes())?;
    let service_key = hmac_sha256(&region_key, b"s3")?;
    let signing_key = hmac_sha256(&service_key, b"aws4_request")?;
    Ok(hex(&hmac_sha256(&signing_key, string_to_sign.as_bytes())?))
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    let mut mac = HmacSha256::new_from_slice(key).map_err(|error| error.to_string())?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn object_key(
    config: &ReporterConfig,
    data: &[u8],
    extension: &str,
    kind: &str,
    label: &str,
) -> String {
    let now = OffsetDateTime::now_utc();
    let sha = sha256_hex(data);
    let template = if config.s3.key_template.trim().is_empty() {
        "{kind}/{Y}/{M}/{D}/{SHA}.{ext}"
    } else {
        config.s3.key_template.trim()
    };

    template
        .replace("{kind}", kind)
        .replace("{Y}", &format!("{:04}", now.year()))
        .replace("{M}", &format!("{:02}", u8::from(now.month())))
        .replace("{D}", &format!("{:02}", now.day()))
        .replace("{SHA}", &sha)
        .replace("{sha}", &sha)
        .replace("{ext}", extension)
        .replace("{APP}", &safe_path_segment(label))
        .replace("{app}", &safe_path_segment(&label.to_lowercase()))
        .trim_start_matches('/')
        .to_string()
}

fn sha256_hex(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn endpoint(config: &ReporterConfig) -> String {
    if config.s3.endpoint.trim().is_empty() {
        format!(
            "https://{}.s3.{}.amazonaws.com",
            config.s3.bucket.trim(),
            config.s3.region.trim()
        )
    } else {
        config.s3.endpoint.trim().trim_end_matches('/').to_string()
    }
}

fn public_url(config: &ReporterConfig, object_key: &str) -> String {
    if !config.s3.custom_domain.trim().is_empty() {
        return format!(
            "{}/{}",
            config.s3.custom_domain.trim().trim_end_matches('/'),
            object_key
        );
    }

    format!(
        "{}/{}/{}",
        endpoint(config),
        config.s3.bucket.trim(),
        object_key
    )
}

fn amz_date() -> String {
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

fn http_date(date: OffsetDateTime) -> String {
    let weekday = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        [date.weekday().number_days_from_monday() as usize];
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][u8::from(date.month()) as usize - 1];
    format!(
        "{weekday}, {:02} {month} {:04} {:02}:{:02}:{:02} GMT",
        date.day(),
        date.year(),
        date.hour(),
        date.minute(),
        date.second()
    )
}

fn extension_from_mime(mime_type: &str) -> &'static str {
    match mime_type {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        _ => "png",
    }
}

fn safe_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    sanitized.trim_matches('-').to_string()
}
