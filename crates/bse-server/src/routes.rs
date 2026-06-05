//! Router assembly.
//!
//! Builds the single [`axum::Router`] mounted by the server. Keeping
//! this in one place makes it trivial to spin up the same router from
//! integration tests via `tower::ServiceExt::oneshot`.

use axum::{Router, routing::get};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::handlers::{health::health, info::info};
use crate::ws::ws_room;

/// Build the application router with all v008 routes wired up.
///
/// Routes :
/// - `GET /health` — liveness probe.
/// - `GET /api/info` — server identity and protocol version.
/// - `GET /ws/rooms/:room_id` — WebSocket upgrade (echo loop).
///
/// Middleware :
/// - `TraceLayer` — per-request structured logs.
/// - `CorsLayer::permissive` — allow any origin during development.
pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/info", get(info))
        .route("/ws/rooms/:room_id", get(ws_room))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
