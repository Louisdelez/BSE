//! WebSocket upgrade handler with per-room broadcast (v010.2),
//! optional JWT verification (v016.2), and snapshot replay (v018).
//!
//! Each connection joins the [`crate::rooms::RoomManager`] on upgrade.
//! Inbound binary / text frames are fanned out to every other peer in
//! the same room. Right after joining, the peer is sent the latest
//! persisted snapshot via [`ServerMessage::Snapshot`] so it does not
//! see an empty canvas when joining an active room.

use axum::{
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bse_auth::TokenType;
use bse_protocol::{OpPayload, ServerMessage};
use bse_types::UserId;
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
        let claims = match app.jwt.verify(token) {
            Ok(c) if c.token_type == TokenType::Access => c,
            Ok(_) => {
                warn!(%room_id, "ws upgrade rejected : refresh token presented");
                return (StatusCode::UNAUTHORIZED, "wrong token type").into_response();
            }
            Err(err) => {
                warn!(%room_id, error = %err, "ws upgrade rejected : invalid token");
                return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
            }
        };
        let Ok(user_id) = claims.sub.parse::<UserId>() else {
            warn!(%room_id, "ws upgrade rejected : malformed user id");
            return (StatusCode::UNAUTHORIZED, "invalid subject").into_response();
        };
        debug!(%room_id, %user_id, "ws upgrade authenticated");

        // v021 : membership check.
        // - user is already a member → allow.
        // - room does not exist yet → auto-create it with the user as
        //   owner (keeps the "set BSE_ROOM and connect" UX working).
        // - room exists but user is not a member → 403.
        match app.store.role_of(&room_id, user_id) {
            Ok(Some(_)) => {}
            Ok(None) => match app.store.room_exists(&room_id) {
                Ok(true) => {
                    warn!(%room_id, %user_id, "ws upgrade rejected : not a member");
                    return (StatusCode::FORBIDDEN, "not a member of this room").into_response();
                }
                Ok(false) => {
                    if let Err(err) = app.store.create_room(&room_id, &room_id, user_id) {
                        warn!(%room_id, error = %err, "auto-create room failed");
                        return (StatusCode::INTERNAL_SERVER_ERROR, "room init failed")
                            .into_response();
                    }
                    info!(%room_id, %user_id, "auto-created room as owner on first WS connect");
                }
                Err(err) => {
                    warn!(%room_id, error = %err, "room_exists check failed");
                    return (StatusCode::INTERNAL_SERVER_ERROR, "room lookup failed")
                        .into_response();
                }
            },
            Err(err) => {
                warn!(%room_id, error = %err, "membership lookup failed");
                return (StatusCode::INTERNAL_SERVER_ERROR, "membership lookup failed")
                    .into_response();
            }
        }
    }

    info!(%room_id, "websocket upgrade requested");
    ws.on_upgrade(move |socket| handle_socket(app, socket, room_id))
        .into_response()
}

/// Per-connection task : join the room, replay the latest persisted
/// snapshot if any, then loop on `socket.recv()` / `inbox.recv()`.
async fn handle_socket(app: AppState, mut socket: WebSocket, room_id: String) {
    let (conn_id, mut inbox) = app.rooms.join(&room_id).await;
    info!(%room_id, connection_id = conn_id.0, "websocket connected");

    // v018 : replay the most recent snapshot to the joining peer so it
    // doesn't see an empty canvas.
    if let Some(bytes) = app.rooms.snapshot(&room_id).await {
        let msg = ServerMessage::Snapshot(OpPayload { seq: 0, bytes });
        match rmp_serde::to_vec_named(&msg) {
            Ok(encoded) => {
                if let Err(err) = socket.send(Message::Binary(encoded)).await {
                    warn!(%room_id, error = %err, "failed to send replay snapshot");
                } else {
                    debug!(%room_id, "replay snapshot sent to joining peer");
                }
            }
            Err(err) => warn!(%room_id, error = %err, "encode replay snapshot failed"),
        }
    }

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
