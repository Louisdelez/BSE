//! Router assembly.
//!
//! Builds the single [`axum::Router`] mounted by the server. The router
//! is fully decoupled from the binary's runtime concerns (signal
//! handling, listener) so integration tests can exercise it via
//! `tower::ServiceExt::oneshot`.

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::handlers::{
    auth::{login, register},
    health::health,
    info::info,
};
use crate::state::AppState;
use crate::ws::ws_room;

/// Build the application router with all current routes wired up.
///
/// Routes :
/// - `GET  /health`               — liveness probe.
/// - `GET  /api/info`             — server identity and protocol version.
/// - `POST /api/auth/login`       — verify credentials, return JWTs (v016.1).
/// - `POST /api/auth/register`    — create a new user (v016.1).
/// - `GET  /ws/rooms/:room_id`    — WebSocket upgrade with per-room
///   broadcast (v010.2).
///
/// Middleware :
/// - `TraceLayer` — per-request structured logs.
/// - `CorsLayer::permissive` — allow any origin during development.
pub fn router_with(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/info", get(info))
        .route("/api/auth/login", post(login))
        .route("/api/auth/register", post(register))
        .route("/ws/rooms/:room_id", get(ws_room))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Convenience wrapper that builds the default [`AppState`] and the
/// router in one call. Useful for tests and the main binary.
pub fn router() -> Router {
    router_with(AppState::build())
}
