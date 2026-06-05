//! `GET /health` — liveness probe.

use axum::{Json, response::IntoResponse};
use serde_json::json;

/// Respond with `{"status": "ok"}`.
///
/// Used by load balancers and tests to confirm the process is alive
/// and the HTTP listener is accepting requests.
pub async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn returns_status_ok_json() {
        let resp = health().await.into_response();
        assert_eq!(resp.status(), 200);
        let body = to_bytes(resp.into_body(), 1024).await.expect("read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(json["status"], "ok");
    }
}
