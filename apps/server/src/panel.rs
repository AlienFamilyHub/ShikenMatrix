use axum::Json;
use axum::body::Body;
use axum::extract::{OriginalUri, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::{Stream, stream};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::{ClientSnapshot, SharedDashboardState};
use crate::storage::{AccessSettings, UpstreamSettings};

#[derive(RustEmbed)]
#[folder = "../panel/dist"]
struct PanelAsset;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

pub async fn api_login(
    State(state): State<SharedDashboardState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    if state
        .storage()
        .verify_user(&payload.username, &payload.password)
    {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + 30 * 24 * 3600; // 30 days
        let claims = Claims {
            sub: payload.username,
            exp: expiration,
        };
        let secret = state.storage().get_jwt_secret();
        match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        ) {
            Ok(token) => (
                StatusCode::OK,
                Json(serde_json::json!(LoginResponse { token })),
            )
                .into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "token generation failed" })),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "invalid username or password" })),
        )
            .into_response()
    }
}

pub async fn api_state(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "admin token required" })),
        )
            .into_response();
    }
    axum::Json(state.snapshot()).into_response()
}

pub async fn api_share_desktop(State(state): State<SharedDashboardState>) -> impl IntoResponse {
    axum::Json(state.desktop_share_snapshot())
}

pub async fn api_share_mobile(State(state): State<SharedDashboardState>) -> impl IntoResponse {
    axum::Json(state.mobile_share_snapshot())
}

pub async fn api_share_desktop_events(
    State(state): State<SharedDashboardState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    share_events(state, "desktop", |state| state.desktop_share_snapshot())
}

pub async fn api_share_mobile_events(
    State(state): State<SharedDashboardState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    share_events(state, "mobile", |state| state.mobile_share_snapshot())
}

#[derive(Deserialize)]
pub struct AssetQuery {
    id: String,
}

pub async fn api_asset(
    State(state): State<SharedDashboardState>,
    Query(query): Query<AssetQuery>,
) -> Response {
    let Some(asset) = state.cached_asset(&query.id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(data) = std::fs::read(&asset.path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&asset.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );

    (headers, data).into_response()
}

fn share_events(
    state: SharedDashboardState,
    event_name: &'static str,
    snapshot: impl Fn(&SharedDashboardState) -> ClientSnapshot + Clone + Send + 'static,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.subscribe_share_updates();
    let events = stream::unfold(
        (state, receiver, true, snapshot),
        move |(state, mut receiver, is_initial, snapshot)| async move {
            if !is_initial && receiver.changed().await.is_err() {
                return None;
            }

            let Ok(data) = serde_json::to_string(&snapshot(&state)) else {
                return None;
            };

            Some((
                Ok(Event::default().event(event_name).data(data)),
                (state, receiver, false, snapshot),
            ))
        },
    );

    Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn api_get_upstream(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "admin token required" })),
        )
            .into_response();
    }
    Json(state.upstream_settings()).into_response()
}

pub async fn api_save_upstream(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
    Json(settings): Json<UpstreamSettings>,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "invalid admin token" })),
        )
            .into_response();
    }

    match state.save_upstream_settings(settings) {
        Ok(()) => Json(state.upstream_settings()).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error })),
        )
            .into_response(),
    }
}

pub async fn api_get_access(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "admin token required" })),
        )
            .into_response();
    }
    Json(state.access_settings()).into_response()
}

pub async fn api_save_access(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
    Json(settings): Json<AccessSettings>,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "admin token required" })),
        )
            .into_response();
    }
    // Clamp the activity log limit to a sane range.
    let mut settings = settings;
    if settings.activity_log_limit == 0 {
        settings.activity_log_limit = 120;
    }
    settings.activity_log_limit = settings.activity_log_limit.clamp(1, 10_000);
    match state.save_access_settings(settings.clone()) {
        Ok(()) => Json(state.access_settings()).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

pub async fn api_change_password(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let username = match admin_username(&state, &headers) {
        Some(username) => username,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "admin token required" })),
            )
                .into_response();
        }
    };
    match state.storage().change_password(
        &username,
        &payload.current_password,
        &payload.new_password,
    ) {
        Ok(()) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": error })),
        )
            .into_response(),
    }
}

#[derive(Serialize)]
struct DataResponse {
    total_events: u64,
    total_messages: u64,
    window_info_count: u64,
    media_playback_count: u64,
    artwork_uploads: u64,
    upstream_errors: u64,
}

pub async fn api_get_data(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "admin token required" })),
        )
            .into_response();
    }
    let snapshot = state.snapshot();
    Json(DataResponse {
        total_events: state.storage().count_activity(),
        total_messages: snapshot.stats.total_messages,
        window_info_count: snapshot.stats.window_info_count,
        media_playback_count: snapshot.stats.media_playback_count,
        artwork_uploads: snapshot.stats.artwork_uploads,
        upstream_errors: snapshot.stats.upstream_errors,
    })
    .into_response()
}

pub async fn api_clear_activity(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "admin token required" })),
        )
            .into_response();
    }
    match state.clear_activity_log() {
        Ok(()) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error })),
        )
            .into_response(),
    }
}

pub async fn api_reset_stats(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "admin token required" })),
        )
            .into_response();
    }
    match state.reset_stats() {
        Ok(()) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct CreateClientKeyRequest {
    description: String,
}

pub async fn api_get_client_keys(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "unauthorized" })),
        )
            .into_response();
    }
    Json(state.storage().get_client_keys()).into_response()
}

pub async fn api_create_client_key(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
    Json(payload): Json<CreateClientKeyRequest>,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "unauthorized" })),
        )
            .into_response();
    }
    match state.storage().create_client_key(&payload.description) {
        Ok(key) => Json(serde_json::json!({ "api_key": key })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

pub async fn api_delete_client_key(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "unauthorized" })),
        )
            .into_response();
    }
    match state.storage().delete_client_key(id) {
        Ok(_) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    uptime_seconds: u64,
    bind_addr: String,
}

pub async fn api_health(State(state): State<SharedDashboardState>) -> impl IntoResponse {
    let snapshot = state.snapshot();
    axum::Json(HealthResponse {
        status: "ok",
        uptime_seconds: snapshot.uptime_seconds,
        bind_addr: snapshot.bind_addr,
    })
}

fn is_admin_authorized(state: &SharedDashboardState, headers: &HeaderMap) -> bool {
    admin_username(state, headers).is_some()
}

fn admin_username(state: &SharedDashboardState, headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?;
    let auth_str = value.to_str().ok()?;
    if !auth_str.starts_with("Bearer ") {
        return None;
    }
    let token = &auth_str[7..];

    let secret = state.storage().get_jwt_secret();
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims.sub)
}

pub async fn panel_fallback(OriginalUri(uri): OriginalUri) -> Response {
    let path = uri.path();
    if path.starts_with("/api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let relative = path.trim_start_matches('/');
    if relative.is_empty() {
        serve_asset("index.html")
    } else {
        serve_asset(relative)
    }
}

fn serve_asset(path: &str) -> Response {
    match PanelAsset::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let cache = if path == "index.html" {
                "no-cache"
            } else {
                "public, max-age=31536000, immutable"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, cache)
                .body(Body::from(file.data.to_vec()))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None if path != "index.html" => serve_asset("index.html"),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
