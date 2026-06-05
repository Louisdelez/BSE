//! Local cursor tracker that throttles outgoing awareness updates to 30 Hz.
//!
//! The UI layer should call [`LocalCursor::maybe_emit`] every frame with
//! the current cursor world position ; the helper returns `Some(bytes)`
//! only when both conditions are met :
//!
//! 1. The position **changed** since the last emitted value.
//! 2. At least `33 ms` elapsed since the last emission.
//!
//! The returned bytes are MessagePack-encoded [`CursorState`] and can be
//! shipped verbatim inside an [`bse_protocol::AwarenessPayload`].

use bse_types::Vec2;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

/// Minimum interval between two cursor emissions (≈ 30 Hz).
const MIN_INTERVAL_MS: u64 = 33;

/// Wire shape for a cursor awareness payload.
///
/// Kept deliberately small : two `f32` add up to 9 `MessagePack` bytes
/// including the map header, which is well below the typical MTU.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) struct CursorState {
    /// Cursor X in world coordinates.
    pub x: f32,
    /// Cursor Y in world coordinates.
    pub y: f32,
}

/// Throttler for outgoing local-cursor awareness updates.
///
/// Drop and recreate the helper if the local user "leaves" and "rejoins"
/// the room : there is no `reset` method on purpose, to keep the type
/// stateless from the outside.
#[derive(Debug)]
pub struct LocalCursor {
    last_position: Option<Vec2>,
    last_emit_at: Option<Instant>,
}

impl LocalCursor {
    /// Construct a fresh tracker. The first call to [`Self::maybe_emit`]
    /// will always emit (since no previous position is recorded).
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_position: None,
            last_emit_at: None,
        }
    }

    /// Returns `Some(bytes-to-send)` when the cursor moved **and** at least
    /// 33 ms passed since the last emission ; otherwise returns `None`.
    ///
    /// `position` is in **world** coordinates (post-camera transform).
    #[must_use]
    pub fn maybe_emit(&mut self, position: Vec2) -> Option<Vec<u8>> {
        let now = Instant::now();
        let moved = self.last_position != Some(position);

        if !moved {
            return None;
        }

        if let Some(prev) = self.last_emit_at {
            let elapsed = now.duration_since(prev);
            if elapsed.as_millis() < u128::from(MIN_INTERVAL_MS) {
                return None;
            }
        }

        let bytes = rmp_serde::to_vec(&CursorState {
            x: position.x,
            y: position.y,
        })
        .ok()?;

        self.last_position = Some(position);
        self.last_emit_at = Some(now);
        Some(bytes)
    }
}

impl Default for LocalCursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test(start_paused = true)]
    async fn first_call_emits() {
        let mut cursor = LocalCursor::new();
        let bytes = cursor.maybe_emit(Vec2::new(1.0, 2.0));
        assert!(bytes.is_some(), "first call should always emit");
    }

    #[tokio::test(start_paused = true)]
    async fn same_position_is_dropped() {
        let mut cursor = LocalCursor::new();
        let _ = cursor.maybe_emit(Vec2::new(5.0, 5.0));
        tokio::time::advance(Duration::from_millis(100)).await;
        // Same position : nothing to send even after the throttle window.
        assert!(cursor.maybe_emit(Vec2::new(5.0, 5.0)).is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn throttle_window_blocks_emission() {
        let mut cursor = LocalCursor::new();
        let _ = cursor.maybe_emit(Vec2::new(0.0, 0.0));
        tokio::time::advance(Duration::from_millis(10)).await;
        // Moved, but within the 33 ms window : suppressed.
        assert!(cursor.maybe_emit(Vec2::new(1.0, 1.0)).is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn emits_again_after_window() {
        let mut cursor = LocalCursor::new();
        let _ = cursor.maybe_emit(Vec2::new(0.0, 0.0));
        tokio::time::advance(Duration::from_millis(40)).await;
        let bytes = cursor.maybe_emit(Vec2::new(2.0, 3.0));
        assert!(bytes.is_some(), "should re-emit after 33 ms");
    }

    #[tokio::test(start_paused = true)]
    async fn payload_roundtrips_via_msgpack() {
        let mut cursor = LocalCursor::new();
        let bytes = cursor
            .maybe_emit(Vec2::new(-1.5, 7.25))
            .expect("first call emits");
        let decoded: CursorState = rmp_serde::from_slice(&bytes).expect("valid msgpack");
        assert!((decoded.x - -1.5).abs() < f32::EPSILON);
        assert!((decoded.y - 7.25).abs() < f32::EPSILON);
    }
}
