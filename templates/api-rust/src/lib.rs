use std::env;
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Deserialize)]
struct EchoIn {
    message: String,
}

#[derive(Serialize)]
struct EchoOut {
    message: String,
}

pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/echo", post(echo))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

async fn echo(Json(body): Json<EchoIn>) -> Result<Json<EchoOut>, StatusCode> {
    if body.message.is_empty() || body.message.len() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(Json(EchoOut {
        message: body.message,
    }))
}

fn cors_layer() -> CorsLayer {
    let production = env::var("ENV").unwrap_or_default() == "production";
    let raw = env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8000,http://127.0.0.1:8000".to_string());
    let origins: Vec<HeaderValue> = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "*")
        .filter_map(|value| value.parse().ok())
        .collect();

    if production && origins.is_empty() {
        return CorsLayer::new();
    }

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

#[cfg(test)]
mod tests {
    use super::app;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_does_not_leak_config() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(text, "{\"status\":\"ok\"}");
        assert!(!text.to_lowercase().contains("secret"));
    }

    #[tokio::test]
    async fn echo_rejects_empty_message() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/echo")
                    .header("content-type", "application/json")
                    .body(Body::from("{\"message\":\"\"}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
