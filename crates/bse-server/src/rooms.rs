//! Per-room broadcast registry.
//!
//! Each room maintains a small map `connection_id -> sender_channel`.
//! Inbound messages on one connection are fanned out to every other
//! connection in the same room. Disconnects clean up automatically.
//!
//! Strategy is intentionally simple : one `tokio::sync::broadcast` per
//! room would also work, but the `connection_id` keys let us skip
//! self-echo cheaply, and they will be needed when v017 introduces
//! per-peer auth / role checks.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use tokio::sync::{Mutex, mpsc};
use tracing::debug;

/// Identifier for a single WebSocket connection inside a room.
///
/// Allocated by the [`RoomManager`] on join. Wraps `u64` ; uniqueness is
/// scoped to the lifetime of one server process which is enough.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ConnectionId(pub u64);

/// Per-room state, kept under a mutex.
#[derive(Default)]
struct RoomState {
    next_id: u64,
    connections: HashMap<ConnectionId, mpsc::UnboundedSender<Message>>,
}

impl RoomState {
    fn allocate(&mut self) -> ConnectionId {
        let id = ConnectionId(self.next_id);
        self.next_id += 1;
        id
    }
}

/// Process-global room registry.
///
/// `Clone`-able so it can be stored in `axum::extract::State` and
/// shared across all WebSocket handlers.
#[derive(Clone, Default)]
pub struct RoomManager {
    rooms: Arc<Mutex<HashMap<String, RoomState>>>,
}

impl RoomManager {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new connection in `room_id`. Returns the freshly
    /// allocated [`ConnectionId`] and a receiver that yields every
    /// message that should be sent to *this* peer.
    pub async fn join(
        &self,
        room_id: &str,
    ) -> (ConnectionId, mpsc::UnboundedReceiver<Message>) {
        let (tx, rx) = mpsc::unbounded_channel::<Message>();
        let mut rooms = self.rooms.lock().await;
        let room = rooms.entry(room_id.to_string()).or_default();
        let id = room.allocate();
        room.connections.insert(id, tx);
        debug!(
            target: "bse::server::rooms",
            %room_id,
            connection_id = id.0,
            "peer joined",
        );
        (id, rx)
    }

    /// Remove a connection from `room_id`. The empty room is kept in
    /// the map ; rooms are not auto-pruned in v010.2 (cheap).
    pub async fn leave(&self, room_id: &str, conn: ConnectionId) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.connections.remove(&conn);
            debug!(
                target: "bse::server::rooms",
                %room_id,
                connection_id = conn.0,
                remaining = room.connections.len(),
                "peer left",
            );
        }
    }

    /// Fan-out a message to every connection in `room_id` *except*
    /// `sender`. Connections whose receiver has been dropped are
    /// silently skipped (the cleanup happens on the next [`leave`]).
    pub async fn broadcast(&self, room_id: &str, sender: ConnectionId, msg: &Message) {
        let rooms = self.rooms.lock().await;
        let Some(room) = rooms.get(room_id) else {
            return;
        };
        for (id, tx) in &room.connections {
            if *id == sender {
                continue;
            }
            // Silently ignore send errors : the worker will be reaped
            // on its own via `leave`.
            let _ = tx.send(msg.clone());
        }
    }

    /// Number of currently-connected peers in `room_id` (0 if room
    /// does not exist).
    pub async fn peer_count(&self, room_id: &str) -> usize {
        self.rooms
            .lock()
            .await
            .get(room_id)
            .map_or(0, |r| r.connections.len())
    }
}
