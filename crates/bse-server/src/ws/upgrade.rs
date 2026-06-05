//! WebSocket upgrade handler and per-connection echo loop.

use axum::{
    extract::{
        Path,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use tracing::{debug, info, warn};

/// Upgrade an HTTP request to a WebSocket bound to a given room.
///
/// The `room_id` path segment is captured but, in v008, it is only
/// used for logging. The real room registry arrives in v009.
pub async fn ws_room(ws: WebSocketUpgrade, Path(room_id): Path<String>) -> impl IntoResponse {
    info!(%room_id, "websocket upgrade requested");
    ws.on_upgrade(move |socket| handle_socket(socket, room_id))
}

/// Per-connection task : echo every Text/Binary frame back, then quit
/// when the peer sends Close or the transport errors out.
async fn handle_socket(mut socket: WebSocket, room_id: String) {
    info!(%room_id, "websocket connected");

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(err) => {
                warn!(%room_id, error = %err, "websocket receive error, closing");
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                debug!(%room_id, bytes = text.len(), "echo text");
                if let Err(err) = socket.send(Message::Text(text)).await {
                    warn!(%room_id, error = %err, "websocket send error, closing");
                    break;
                }
            }
            Message::Binary(bin) => {
                debug!(%room_id, bytes = bin.len(), "echo binary");
                if let Err(err) = socket.send(Message::Binary(bin)).await {
                    warn!(%room_id, error = %err, "websocket send error, closing");
                    break;
                }
            }
            Message::Ping(_) | Message::Pong(_) => {
                // axum auto-replies to Ping with Pong; nothing to do.
            }
            Message::Close(frame) => {
                info!(%room_id, ?frame, "websocket close frame received");
                break;
            }
        }
    }

    info!(%room_id, "websocket disconnected");
}
