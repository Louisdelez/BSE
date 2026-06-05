//! WebSocket client wrapping `tokio_tungstenite::WebSocketStream`.
//!
//! The client speaks the [`bse_protocol`] wire format over a single
//! WebSocket. Frames are encoded as `MessagePack` and sent as binary
//! WebSocket messages.
//!
//! # Reconnection policy
//!
//! This v010 client does **not** reconnect on its own. The caller owns
//! the connection lifecycle : on disconnection, drop the client and
//! call [`SyncClient::connect`] again. A higher-level "session" wrapper
//! with exponential backoff is planned for a later milestone.

use bse_protocol::{AwarenessPayload, ClientMessage, HelloPayload, OpPayload, ServerMessage};
use bse_types::PeerId;
use futures_util::{FutureExt, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{info, warn};
use url::Url;

use crate::error::SyncError;
use crate::state::ConnectionState;

/// Type alias for the concrete WebSocket stream we wrap.
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Static client version reported in the `Hello` handshake.
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parameters required to open a sync connection.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// Base URL of the server, **without** the room path component.
    ///
    /// Examples : `"ws://localhost:8080"`, `"wss://bse.example.com"`.
    pub server_url: String,
    /// Identifier of the room to join.
    pub room_id: String,
    /// Stable identifier of the local peer.
    pub peer_id: PeerId,
    /// Display name used by other peers when rendering awareness.
    pub display_name: String,
    /// Optional JWT access token passed as `?token=...` on the WS URL.
    ///
    /// Required when the server is started with `BSE_REQUIRE_AUTH=1` ;
    /// ignored otherwise. The token is verified by the server before the
    /// WebSocket upgrade completes — an invalid token yields a 401.
    pub auth_token: Option<String>,
}

/// Live WebSocket connection to a BSE room.
///
/// The struct is **not** `Clone` : owning it means you own the underlying
/// socket. To share it across tasks, wrap it in `Arc<Mutex<_>>` or split
/// the stream upstream of this crate.
pub struct SyncClient {
    stream: WsStream,
    state: ConnectionState,
    config: ClientConfig,
}

impl SyncClient {
    /// Connect to `{server_url}/ws/rooms/{room_id}` and send the initial
    /// [`ClientMessage::Hello`] handshake frame.
    ///
    /// The v008 server is echo-only, so we do **not** wait for a
    /// [`ServerMessage::Welcome`]. The handshake is fire-and-forget at
    /// this milestone ; later versions will block until `Welcome` (or a
    /// timeout) before returning.
    pub async fn connect(config: ClientConfig) -> Result<Self, SyncError> {
        let url = build_ws_url(
            &config.server_url,
            &config.room_id,
            config.auth_token.as_deref(),
        )?;
        info!(url = %url, room_id = %config.room_id, "sync client connecting");

        let (stream, _response) = connect_async(url.as_str())
            .await
            .map_err(|e| SyncError::Connection(format!("ws upgrade : {e}")))?;

        let mut client = Self {
            stream,
            state: ConnectionState::Connecting,
            config,
        };

        client.send_hello().await?;
        client.state = ConnectionState::Connected;
        info!(peer_id = %client.config.peer_id, "sync client connected");

        Ok(client)
    }

    /// Snapshot of the current connection state.
    #[must_use]
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Send a locally-produced CRDT op to the server.
    pub async fn send_op(&mut self, op: OpPayload) -> Result<(), SyncError> {
        self.send_client_message(&ClientMessage::Op(op)).await
    }

    /// Send an awareness update (typically the local cursor).
    pub async fn send_awareness(&mut self, payload: AwarenessPayload) -> Result<(), SyncError> {
        self.send_client_message(&ClientMessage::Awareness(payload))
            .await
    }

    /// Receive the next server message, if any is immediately available.
    ///
    /// Returns :
    /// - `Ok(Some(msg))` if a complete protocol frame was decoded.
    /// - `Ok(None)` if the stream is open but no frame is pending right
    ///   now (the caller can yield and try again later).
    /// - `Err(_)` if the transport failed or a frame could not be decoded.
    ///
    /// Control frames (`Ping`/`Pong`/`Close`) and unexpected text frames
    /// are handled transparently and return `Ok(None)`.
    #[allow(clippy::unused_async)] // kept async for symmetry with the rest of the public API
    pub async fn next_message(&mut self) -> Result<Option<ServerMessage>, SyncError> {
        // `now_or_never` polls the future exactly once. If it is not
        // ready, we return `Ok(None)` without blocking.
        let Some(next) = self.stream.next().now_or_never() else {
            return Ok(None);
        };

        let Some(frame) = next else {
            // Stream ended.
            self.state = ConnectionState::Offline;
            return Err(SyncError::Connection("stream closed by peer".into()));
        };

        let frame = frame.map_err(|e| SyncError::Connection(format!("ws recv : {e}")))?;

        match frame {
            Message::Binary(bytes) => {
                let msg: ServerMessage = rmp_serde::from_slice(&bytes)
                    .map_err(|e| SyncError::Protocol(format!("server msg decode : {e}")))?;
                Ok(Some(msg))
            }
            Message::Text(text) => {
                // v008 echoes our Hello back as binary, but a hostile
                // server might send text. We log and ignore.
                warn!(bytes = text.len(), "unexpected text frame, ignoring");
                Ok(None)
            }
            Message::Close(frame) => {
                info!(?frame, "server sent close frame");
                self.state = ConnectionState::Offline;
                Ok(None)
            }
            Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => Ok(None),
        }
    }

    /// Close the connection cleanly with a normal-closure frame.
    pub async fn close(mut self) -> Result<(), SyncError> {
        info!(peer_id = %self.config.peer_id, "sync client closing");
        self.stream
            .close(None)
            .await
            .map_err(|e| SyncError::Connection(format!("ws close : {e}")))?;
        self.state = ConnectionState::Offline;
        Ok(())
    }

    /// Encode `msg` as `MessagePack` and send it as a binary WS frame.
    async fn send_client_message(&mut self, msg: &ClientMessage) -> Result<(), SyncError> {
        let bytes = rmp_serde::to_vec_named(msg)
            .map_err(|e| SyncError::Protocol(format!("client msg encode : {e}")))?;
        self.stream
            .send(Message::Binary(bytes))
            .await
            .map_err(|e| SyncError::Connection(format!("ws send : {e}")))?;
        Ok(())
    }

    async fn send_hello(&mut self) -> Result<(), SyncError> {
        let hello = ClientMessage::Hello(HelloPayload {
            peer_id: self.config.peer_id,
            client_version: CLIENT_VERSION.to_string(),
            last_seen_op: None,
        });
        self.send_client_message(&hello).await
    }
}

/// Build the full `ws[s]://host/ws/rooms/{room_id}` URL, optionally
/// appending `?token=...` when `auth_token` is provided.
fn build_ws_url(
    server_url: &str,
    room_id: &str,
    auth_token: Option<&str>,
) -> Result<Url, SyncError> {
    let base = Url::parse(server_url)
        .map_err(|e| SyncError::Connection(format!("invalid server URL : {e}")))?;

    match base.scheme() {
        "ws" | "wss" => {}
        other => {
            return Err(SyncError::Connection(format!(
                "unsupported scheme `{other}`, expected ws or wss",
            )));
        }
    }

    let path = format!("/ws/rooms/{room_id}");
    let mut url = base
        .join(&path)
        .map_err(|e| SyncError::Connection(format!("URL join : {e}")))?;
    if let Some(token) = auth_token
        && !token.is_empty()
    {
        url.query_pairs_mut().append_pair("token", token);
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_ws_url_plain() {
        let u = build_ws_url("ws://localhost:8080", "room-42", None).expect("valid");
        assert_eq!(u.scheme(), "ws");
        assert_eq!(u.host_str(), Some("localhost"));
        assert_eq!(u.port(), Some(8080));
        assert_eq!(u.path(), "/ws/rooms/room-42");
    }

    #[test]
    fn build_ws_url_secure() {
        let u = build_ws_url("wss://bse.example.com", "abc", None).expect("valid");
        assert_eq!(u.scheme(), "wss");
        assert_eq!(u.path(), "/ws/rooms/abc");
    }

    #[test]
    fn build_ws_url_rejects_http_scheme() {
        let err = build_ws_url("http://localhost:8080", "x", None)
            .expect_err("http should be rejected as the protocol mismatch is silent otherwise");
        assert!(matches!(err, SyncError::Connection(_)));
    }

    #[test]
    fn build_ws_url_rejects_garbage() {
        let err = build_ws_url("not a url", "x", None).expect_err("garbage rejected");
        assert!(matches!(err, SyncError::Connection(_)));
    }

    // Note : no live-server integration test. The v008 server is an echo
    // loop, so a real connect would receive its own Hello back as the
    // first "server message", which is not a `ServerMessage` and would
    // fail decoding. Once v011 ships real server-side room logic, a
    // round-trip test belongs there, not here.
}
