//! Definition of every message exchanged between client and server.
//!
//! The two top-level enums are [`ClientMessage`] (client → server) and
//! [`ServerMessage`] (server → client). They are flattened with serde
//! using `tag = "t", content = "p"` to keep wire size minimal while
//! remaining self-describing.

use bse_types::PeerId;
use serde::{Deserialize, Serialize};

// ─── Client → Server ─────────────────────────────────────────────────────────

/// Top-level enum for messages sent by a client to the server.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "p")]
pub enum ClientMessage {
    /// Initial handshake message.
    Hello(HelloPayload),
    /// A locally-produced CRDT operation to broadcast.
    Op(OpPayload),
    /// Awareness update (cursor, selection, etc.).
    Awareness(AwarenessPayload),
    /// Request a full snapshot of the current room state.
    RequestSnapshot,
    /// Heartbeat ping.
    Ping(u64),
}

/// Sent by the client right after the WebSocket upgrade.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HelloPayload {
    /// Stable per-device peer identifier.
    pub peer_id: PeerId,
    /// Semver of the client.
    pub client_version: String,
    /// Sequence number of the last op the client has on disk, for
    /// incremental resync. `None` means the client wants a full snapshot.
    pub last_seen_op: Option<u64>,
}

// ─── Server → Client ─────────────────────────────────────────────────────────

/// Top-level enum for messages sent by the server to a client.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "p")]
pub enum ServerMessage {
    /// Response to a `Hello`, sent right after a successful handshake.
    Welcome(WelcomePayload),
    /// Full snapshot of the room state (or an incremental update).
    Snapshot(OpPayload),
    /// A CRDT op produced by another peer, broadcast.
    Op(OpPayload),
    /// An awareness update from another peer.
    Awareness(AwarenessPayload),
    /// A new peer has joined the room.
    PeerJoin(PeerJoinPayload),
    /// A peer has left the room.
    PeerLeave(PeerLeavePayload),
    /// An error occurred ; the server may close the connection after.
    Error(ErrorPayload),
    /// Reply to a `Ping`.
    Pong(u64),
}

/// First message sent by the server after handshake.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WelcomePayload {
    /// Semver of the server.
    pub server_version: String,
    /// Color assigned to this peer for awareness display.
    pub assigned_color: (u8, u8, u8),
    /// Current server-side wall-clock, ms since UNIX epoch.
    pub server_time_ms: u64,
}

// ─── Shared payloads ─────────────────────────────────────────────────────────

/// A binary-encoded CRDT operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpPayload {
    /// Monotonic sequence number, local to the emitting peer.
    pub seq: u64,
    /// Raw bytes of the CRDT op (encoded by the chosen library, opaque here).
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

/// Ephemeral state of a peer (cursor, selection...).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AwarenessPayload {
    /// Peer this update is about.
    pub peer_id: PeerId,
    /// Opaque encoded state ; decoded with the awareness library.
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

/// Notification that a peer has joined the room.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerJoinPayload {
    /// Peer that just joined.
    pub peer_id: PeerId,
    /// Display name of the user behind this peer.
    pub display_name: String,
    /// Color assigned to the peer for awareness display.
    pub color: (u8, u8, u8),
    /// Server-side timestamp of the join, ms since UNIX epoch.
    pub joined_at_ms: u64,
}

/// Notification that a peer has left the room.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerLeavePayload {
    /// Peer that left.
    pub peer_id: PeerId,
}

/// Error reported by the server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Machine-readable error code.
    pub code: ErrorCode,
    /// Human-readable message, suitable for logging.
    pub message: String,
}

/// Categorised error codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// The provided credentials are invalid or missing.
    Unauthorized,
    /// The peer is sending too many ops per second.
    RateLimitExceeded,
    /// The op payload could not be decoded or applied.
    InvalidOp,
    /// The room has reached its maximum number of peers.
    RoomFull,
    /// Generic server-side error ; details are in `message`.
    InternalError,
}
