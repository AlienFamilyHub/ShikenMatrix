use axum::Json;
use axum::body::Body;
use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation};

use crate::state::SharedDashboardState;
use crate::storage::UpstreamSettings;

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
    if state.storage().verify_user(&payload.username, &payload.password) {
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
        match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())) {
            Ok(token) => (StatusCode::OK, Json(serde_json::json!(LoginResponse { token }))).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "token generation failed" }))).into_response()
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "invalid username or password" }))).into_response()
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

pub async fn api_share(State(state): State<SharedDashboardState>) -> impl IntoResponse {
    axum::Json(state.public_snapshot())
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
        Ok(()) => Json(state.snapshot()).into_response(),
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
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "unauthorized" }))).into_response();
    }
    Json(state.storage().get_client_keys()).into_response()
}

pub async fn api_create_client_key(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
    Json(payload): Json<CreateClientKeyRequest>,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "unauthorized" }))).into_response();
    }
    match state.storage().create_client_key(&payload.description) {
        Ok(key) => Json(serde_json::json!({ "api_key": key })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

pub async fn api_delete_client_key(
    State(state): State<SharedDashboardState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    if !is_admin_authorized(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "unauthorized" }))).into_response();
    }
    match state.storage().delete_client_key(id) {
        Ok(_) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))).into_response(),
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
    let Some(value) = headers.get(header::AUTHORIZATION) else {
        return false;
    };
    let Ok(auth_str) = value.to_str() else { return false; };
    if !auth_str.starts_with("Bearer ") { return false; }
    let token = &auth_str[7..];

    let secret = state.storage().get_jwt_secret();
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation
    ).is_ok()
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
