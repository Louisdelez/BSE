//! Per-room broadcast registry + snapshot persistence (v018).
//!
//! In addition to the in-memory fan-out introduced in v010.2, the room
//! manager now persists CRDT snapshots into the
//! [`crate::store::ServerStore`] so :
//!
//! - room state survives a server restart,
//! - a peer joining mid-session receives the latest snapshot it missed
//!   via [`ServerMessage::Snapshot`] (see [`crate::ws::upgrade`]).
//!
//! Strategy is intentionally simple : whenever a peer broadcasts an
//! `Op` payload we decode it as a [`bse_protocol::ClientMessage`] and,
//! if it is an `Op`, replace the room's stored snapshot. v010.3 ships
//! full snapshots per mutation (not deltas), so this remains correct.
//! v023 will switch to incremental updates + periodic consolidation.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use bse_protocol::ClientMessage;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, warn};

use crate::store::ServerStore;

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
#[derive(Clone)]
pub struct RoomManager {
    rooms: Arc<Mutex<HashMap<String, RoomState>>>,
    store: Arc<ServerStore>,
}

impl RoomManager {
    /// Empty registry wrapping `store`.
    #[must_use]
    pub fn new(store: Arc<ServerStore>) -> Self {
        Self {
            rooms: Arc::default(),
            store,
        }
    }

    /// Register a new connection in `room_id`. Returns the freshly
    /// allocated [`ConnectionId`] and a receiver that yields every
    /// message that should be sent to *this* peer.
    pub async fn join(&self, room_id: &str) -> (ConnectionId, mpsc::UnboundedReceiver<Message>) {
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
    /// the map ; rooms are not auto-pruned (cheap).
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
    /// `sender`. If the frame is a binary `ClientMessage::Op` the
    /// snapshot is also persisted to the store. Connections whose
    /// receiver has been dropped are silently skipped.
    pub async fn broadcast(&self, room_id: &str, sender: ConnectionId, msg: &Message) {
        // Persist snapshot opportunistically (fire-and-forget) before
        // fan-out so a slow store does not slow down recipients.
        if let Message::Binary(bytes) = msg
            && let Ok(ClientMessage::Op(op)) = rmp_serde::from_slice::<ClientMessage>(bytes)
        {
            let store = Arc::clone(&self.store);
            let room_id_owned = room_id.to_string();
            let snapshot = op.bytes;
            tokio::task::spawn_blocking(move || {
                if let Err(err) = store.save_room_snapshot(&room_id_owned, &snapshot) {
                    warn!(
                        target: "bse::server::rooms",
                        room_id = %room_id_owned,
                        error = %err,
                        "save_room_snapshot failed",
                    );
                }
            });
        }

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

    /// Load the latest persisted snapshot for `room_id`, if any.
    pub async fn snapshot(&self, room_id: &str) -> Option<Vec<u8>> {
        let store = Arc::clone(&self.store);
        let room_id_owned = room_id.to_string();
        let result = tokio::task::spawn_blocking(move || store.load_room_snapshot(&room_id_owned))
            .await
            .ok()?;
        match result {
            Ok(opt) => opt,
            Err(err) => {
                warn!(
                    target: "bse::server::rooms",
                    %room_id,
                    error = %err,
                    "load_room_snapshot failed",
                );
                None
            }
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
