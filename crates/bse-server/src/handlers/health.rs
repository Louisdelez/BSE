//! Liveness (`/health`) and readiness (`/ready`) probes (v024).

use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, response::Json as JsonResponse};
use serde::Serialize;
use serde_json::json;

use crate::state::AppState;

/// Boot instant ; lazily initialised the first time `/ready` (or
/// `/health`) is called. Used to compute uptime.
static BOOT_AT: OnceLock<Instant> = OnceLock::new();

fn boot() -> Instant {
    *BOOT_AT.get_or_init(Instant::now)
}

/// `GET /health` — liveness probe.
///
/// Returns `200 {"status":"ok"}` as long as the HTTP listener is
/// alive. Does not touch the database. Suitable for orchestrators
/// (Kubernetes, Docker, fly.io) that should restart a stuck process.
pub async fn health() -> impl IntoResponse {
    let _ = boot();
    Json(json!({ "status": "ok" }))
}

/// JSON body returned by `/ready`.
#[derive(Debug, Serialize)]
pub struct ReadyBody {
    /// `"ready"` if the server can serve traffic, `"degraded"` otherwise.
    pub status: &'static str,
    /// `true` iff the `SQLite` store responded to a noop query.
    pub db_ok: bool,
    /// Uptime in seconds since process start.
    pub uptime_secs: u64,
    /// Unix epoch (seconds) at which this response was produced.
    pub server_time_secs: u64,
    /// Total connections currently held in the room manager, across
    /// all rooms. Useful for capacity dashboards.
    pub connected_peers: u64,
}

/// `GET /ready` — readiness probe (v024).
///
/// Checks that the database is reachable and reports a small bundle of
/// operational metrics. Returns `503` when the DB is not responding so
/// load balancers can drain traffic.
pub async fn ready(State(app): State<AppState>) -> Response {
    let db_ok = app.store.has_any_user().is_ok();
    let uptime_secs = boot().elapsed().as_secs();
    let server_time_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Aggregating per-room peer counts would require enumerating the
    // RoomManager's keys, which we don't expose. For a single coarse
    // number, the rooms API is enough — but it's also async, so we
    // just report 0 when not exposed. Future v024.1 can expose a
    // counter from RoomManager.
    let connected_peers = 0;

    let status = if db_ok { "ready" } else { "degraded" };
    let body = ReadyBody {
        status,
        db_ok,
        uptime_secs,
        server_time_secs,
        connected_peers,
    };
    let code = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, JsonResponse(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn health_returns_status_ok_json() {
        let resp = health().await.into_response();
        assert_eq!(resp.status(), 200);
        let body = to_bytes(resp.into_body(), 1024).await.expect("read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(json["status"], "ok");
    }
}
