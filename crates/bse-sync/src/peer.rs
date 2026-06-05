//! Bookkeeping for remote peers observed through `Awareness` updates.
//!
//! The `SyncClient` itself does **not** maintain a peer registry — that is
//! the caller's job (typically the UI's awareness layer). The types in
//! this module are the canonical shape we recommend storing.

use bse_types::{Color, PeerId, Vec2};

use crate::cursor::CursorState;
use crate::error::SyncError;

/// Snapshot of a single remote peer's awareness state.
///
/// `last_seen` is intentionally a `std::time::Instant` (not `tokio::...`)
/// so the type is usable from non-async UI code.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// Stable identifier of the remote peer.
    pub peer_id: PeerId,
    /// Human-readable name picked up from `PeerJoin`.
    pub display_name: String,
    /// Color assigned by the server for awareness display.
    pub color: Color,
    /// Last known cursor position, in world coordinates.
    ///
    /// `None` until the peer sends its first awareness update.
    pub last_cursor: Option<Vec2>,
    /// Wall-clock instant of the most recent message from this peer.
    pub last_seen: std::time::Instant,
}

impl PeerInfo {
    /// Construct a fresh `PeerInfo` from a `PeerJoin` notification.
    ///
    /// `last_cursor` starts as `None` and is filled in by the first
    /// awareness update.
    #[must_use]
    pub fn new(peer_id: PeerId, display_name: String, color: Color) -> Self {
        Self {
            peer_id,
            display_name,
            color,
            last_cursor: None,
            last_seen: std::time::Instant::now(),
        }
    }

    /// Record a cursor update : refresh both `last_cursor` and `last_seen`.
    pub fn record_cursor(&mut self, position: Vec2) {
        self.last_cursor = Some(position);
        self.last_seen = std::time::Instant::now();
    }
}

/// Decode the byte payload of a cursor `Awareness` message into a [`Vec2`].
///
/// The expected `MessagePack` shape is the one emitted by
/// [`crate::LocalCursor::maybe_emit`].
pub fn decode_cursor(bytes: &[u8]) -> Result<Vec2, SyncError> {
    let state: CursorState = rmp_serde::from_slice(bytes)
        .map_err(|e| SyncError::Protocol(format!("cursor decode : {e}")))?;
    Ok(Vec2::new(state.x, state.y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    // Mirror of `cursor::CursorState` for the encode side : we cannot
    // `use` the crate-private struct directly without going through
    // `LocalCursor`, so we duplicate it here. Both serializations must
    // stay byte-compatible — covered by `cursor::tests::payload_roundtrips_via_msgpack`.
    #[derive(Serialize)]
    struct CursorWire {
        x: f32,
        y: f32,
    }

    #[test]
    fn decode_cursor_roundtrip() {
        let bytes = rmp_serde::to_vec(&CursorWire { x: 12.5, y: -3.0 }).unwrap();
        let v = decode_cursor(&bytes).expect("valid cursor bytes");
        assert!((v.x - 12.5).abs() < f32::EPSILON);
        assert!((v.y - -3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn decode_cursor_garbage_is_protocol_error() {
        let err = decode_cursor(&[0xFF, 0x00, 0xAB]).expect_err("garbage rejected");
        assert!(matches!(err, SyncError::Protocol(_)));
    }

    #[test]
    fn peer_info_records_cursor() {
        let mut info = PeerInfo::new(PeerId::new(), "Alice".into(), Color::BRAND_BLUE);
        assert!(info.last_cursor.is_none());
        let before = info.last_seen;
        std::thread::sleep(std::time::Duration::from_millis(2));
        info.record_cursor(Vec2::new(1.0, 2.0));
        assert_eq!(info.last_cursor, Some(Vec2::new(1.0, 2.0)));
        assert!(info.last_seen > before);
    }
}
