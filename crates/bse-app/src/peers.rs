//! Remote peer tracking — last cursor position, name, color.
//!
//! Populated by [`crate::sync_thread::SyncHandle`] events and rendered
//! by the canvas. Stale peers (no update in `STALE_TIMEOUT`) are evicted
//! to keep the display tidy when peers disconnect without sending
//! `PeerLeave`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use bse_types::{Color, PeerId, Vec2};

/// Drop peers that haven't sent any awareness in this duration.
pub const STALE_TIMEOUT: Duration = Duration::from_secs(20);

/// One remote peer's display state.
#[derive(Clone, Debug)]
pub struct RemotePeer {
    /// Optional display name (only present after a `PeerJoin`).
    pub display_name: Option<String>,
    /// Color assigned by the server for this peer's cursor.
    pub color: Color,
    /// Last cursor position seen in world coordinates.
    pub last_cursor: Option<Vec2>,
    /// Instant of the last received awareness for this peer.
    pub last_seen: Instant,
}

impl RemotePeer {
    /// Build a peer from a `PeerJoin` event.
    #[must_use]
    pub fn joined(display_name: String, color: Color) -> Self {
        Self {
            display_name: Some(display_name),
            color,
            last_cursor: None,
            last_seen: Instant::now(),
        }
    }

    /// Build a peer from an incoming awareness with no prior `PeerJoin`.
    /// Falls back to neutral defaults until a join is seen.
    #[must_use]
    pub fn from_cursor(position: Vec2) -> Self {
        Self {
            display_name: None,
            color: Color::rgb(0xA5, 0xA8, 0xB5),
            last_cursor: Some(position),
            last_seen: Instant::now(),
        }
    }
}

/// In-memory store of remote peers.
#[derive(Default)]
pub struct PeerStore {
    peers: HashMap<PeerId, RemotePeer>,
}

impl PeerStore {
    /// Empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of peers currently displayed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// Record a peer-join event.
    pub fn on_join(&mut self, id: PeerId, display_name: String, color: Color) {
        self.peers
            .entry(id)
            .and_modify(|p| {
                p.display_name = Some(display_name.clone());
                p.color = color;
                p.last_seen = Instant::now();
            })
            .or_insert_with(|| RemotePeer::joined(display_name, color));
    }

    /// Record a peer-leave event.
    pub fn on_leave(&mut self, id: PeerId) {
        self.peers.remove(&id);
    }

    /// Record a cursor update.
    pub fn on_cursor(&mut self, id: PeerId, position: Vec2) {
        self.peers
            .entry(id)
            .and_modify(|p| {
                p.last_cursor = Some(position);
                p.last_seen = Instant::now();
            })
            .or_insert_with(|| RemotePeer::from_cursor(position));
    }

    /// Drop peers that haven't been seen in [`STALE_TIMEOUT`].
    pub fn prune_stale(&mut self) {
        let now = Instant::now();
        self.peers
            .retain(|_, peer| now.duration_since(peer.last_seen) < STALE_TIMEOUT);
    }

    /// Iterate over peers that currently have a known cursor position.
    pub fn with_cursors(&self) -> impl Iterator<Item = (&PeerId, &RemotePeer)> {
        self.peers.iter().filter(|(_, p)| p.last_cursor.is_some())
    }

    /// Drop every peer (e.g. on disconnect).
    pub fn clear(&mut self) {
        self.peers.clear();
    }
}
