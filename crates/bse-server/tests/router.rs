//! Integration tests for the axum router.
//!
//! Exercises the router via `tower::ServiceExt::oneshot` so no real
//! TCP listener is involved.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use bse_protocol::PROTOCOL_VERSION;
use bse_server::router;
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let app = router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router responded");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json");
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn info_endpoint_returns_protocol_version() {
    let app = router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/info")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router responded");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json");
    assert_eq!(json["protocol"], PROTOCOL_VERSION);
    assert_eq!(json["name"], "BSE Server");
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let app = router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/does-not-exist")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router responded");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
