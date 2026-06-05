//! WebSocket upgrade handler with per-room broadcast (v010.2).
//!
//! Each connection is registered in the [`crate::rooms::RoomManager`]
//! on upgrade. Inbound binary / text frames are fanned out to every
//! other peer in the same room, replacing the per-connection echo loop
//! used in v008.
//!
//! When `BSE_REQUIRE_AUTH=1` is set on the server, the upgrade also
//! verifies the JWT passed as `?token=...` on the WebSocket URL
//! before completing the handshake (v016.2). Without that env var the
//! token is ignored and unauthenticated connections are accepted —
//! suitable for local development only.

use axum::{
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bse_auth::TokenType;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::state::AppState;

/// Query-string parameters accepted on `/ws/rooms/{room_id}`.
#[derive(Debug, Deserialize)]
pub struct WsParams {
    /// Optional JWT access token. Required when `BSE_REQUIRE_AUTH=1`.
    #[serde(default)]
    pub token: Option<String>,
}

/// Upgrade an HTTP request to a WebSocket bound to `room_id`.
///
/// The handler is split in two phases :
///
/// 1. *Sync* : optional JWT verification (returns `401` on failure).
/// 2. *Async* : the actual `on_upgrade` task that joins the room and
///    fans messages out.
pub async fn ws_room(
    State(app): State<AppState>,
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    Query(params): Query<WsParams>,
) -> Response {
    if std::env::var("BSE_REQUIRE_AUTH").as_deref() == Ok("1") {
        let Some(token) = params.token.as_deref() else {
            warn!(%room_id, "ws upgrade rejected : missing token");
            return (StatusCode::UNAUTHORIZED, "missing token").into_response();
        };
        match app.jwt.verify(token) {
            Ok(claims) if claims.token_type == TokenType::Access => {
                debug!(%room_id, sub = %claims.sub, "ws upgrade authenticated");
            }
            Ok(_) => {
                warn!(%room_id, "ws upgrade rejected : refresh token presented");
                return (StatusCode::UNAUTHORIZED, "wrong token type").into_response();
            }
            Err(err) => {
                warn!(%room_id, error = %err, "ws upgrade rejected : invalid token");
                return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
            }
        }
    }

    info!(%room_id, "websocket upgrade requested");
    ws.on_upgrade(move |socket| handle_socket(app, socket, room_id))
        .into_response()
}

/// Per-connection task : join the room, then loop on
/// `socket.recv()` / `inbox.recv()`, broadcasting inbound frames to
/// every other peer in the same room.
async fn handle_socket(app: AppState, mut socket: WebSocket, room_id: String) {
    let (conn_id, mut inbox) = app.rooms.join(&room_id).await;
    info!(%room_id, connection_id = conn_id.0, "websocket connected");

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(msg) = incoming else {
                    info!(%room_id, connection_id = conn_id.0, "socket closed by peer");
                    break;
                };
                let msg = match msg {
                    Ok(m) => m,
                    Err(err) => {
                        warn!(%room_id, error = %err, "websocket receive error, closing");
                        break;
                    }
                };
                match msg {
                    Message::Close(frame) => {
                        info!(%room_id, ?frame, "websocket close frame received");
                        break;
                    }
                    Message::Ping(_) | Message::Pong(_) => {}
                    other => {
                        debug!(%room_id, "broadcasting frame to peers");
                        app.rooms.broadcast(&room_id, conn_id, &other).await;
                    }
                }
            }
            outgoing = inbox.recv() => {
                let Some(frame) = outgoing else {
                    debug!(%room_id, "inbox closed");
                    break;
                };
                if let Err(err) = socket.send(frame).await {
                    warn!(%room_id, error = %err, "websocket send error, closing");
                    break;
                }
            }
        }
    }

    app.rooms.leave(&room_id, conn_id).await;
    info!(%room_id, connection_id = conn_id.0, "websocket disconnected");
}
