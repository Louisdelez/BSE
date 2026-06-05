//! WebSocket upgrade handler with per-room broadcast (v010.2).
//!
//! Each connection is registered in the [`crate::rooms::RoomManager`]
//! on upgrade. Inbound binary / text frames are fanned out to every
//! other peer in the same room, replacing the per-connection echo loop
//! used in v008.

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::{IntoResponse, Response},
};
use tracing::{debug, info, warn};

use crate::state::AppState;

/// Upgrade an HTTP request to a WebSocket bound to `room_id`.
pub async fn ws_room(
    State(app): State<AppState>,
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
) -> Response {
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
