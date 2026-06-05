//! Router assembly.

use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::handlers::{
    auth::{login, refresh, register},
    health::health,
    info::info,
    rooms::{add_member, create_room, list_rooms, remove_member},
};
use crate::state::AppState;
use crate::ws::ws_room;

/// Build the application router with all current routes wired up.
///
/// Routes :
/// - `GET    /health`                      — liveness probe.
/// - `GET    /api/info`                    — server identity / protocol version.
/// - `POST   /api/auth/login`              — verify creds, return JWT pair (v016.1).
/// - `POST   /api/auth/register`           — create a new user (v016.1).
/// - `POST   /api/auth/refresh`            — rotate access token via refresh (v020).
/// - `GET    /api/rooms`                   — list my rooms (v021).
/// - `POST   /api/rooms`                   — create a new room (v021).
/// - `POST   /api/rooms/:id/members`       — invite a member (v021, owner-only).
/// - `DELETE /api/rooms/:id/members/:user_id` — kick a member (v021, owner-only).
/// - `GET    /ws/rooms/:room_id`           — WebSocket upgrade with broadcast
///   (v010.2), JWT verification (v016.2), snapshot replay (v018) and room
///   membership check (v021, gated by `BSE_REQUIRE_AUTH=1`).
pub fn router_with(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/info", get(info))
        .route("/api/auth/login", post(login))
        .route("/api/auth/register", post(register))
        .route("/api/auth/refresh", post(refresh))
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/:id/members", post(add_member))
        .route(
            "/api/rooms/:id/members/:user_id",
            delete(remove_member),
        )
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
