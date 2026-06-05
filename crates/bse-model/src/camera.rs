//! The camera defines the mapping between world coordinates (the infinite
//! canvas) and screen coordinates (pixels in the window).
//!
//! A camera is fully described by :
//!
//! - the **world position** of the viewport center, and
//! - a uniform **zoom** factor.
//!
//! Rotation is intentionally not supported in v1 (rare in whiteboard apps
//! and complicates input handling significantly).

use bse_types::{Rect, Vec2};
use serde::{Deserialize, Serialize};

/// Minimum zoom factor (5% — see infinite canvas), enforced by `clamp_zoom`.
pub const MIN_ZOOM: f32 = 0.05;
/// Maximum zoom factor (5000%), enforced by `clamp_zoom`.
pub const MAX_ZOOM: f32 = 50.0;

/// Camera state.
///
/// Not part of the CRDT : each peer has its own camera.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// World-space position of the center of the viewport.
    pub position: Vec2,
    /// Zoom factor (`1.0` is 1:1, `2.0` is 2× zoom in, `0.5` is 2× zoom out).
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Camera {
    /// Camera at origin, zoom 1.
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        zoom: 1.0,
    };

    /// Construct from position and zoom, clamping zoom to the valid range.
    #[must_use]
    pub fn new(position: Vec2, zoom: f32) -> Self {
        Self {
            position,
            zoom: clamp_zoom(zoom),
        }
    }

    /// Convert a screen-space point to world space.
    ///
    /// `viewport` is the physical size of the visible canvas region in pixels.
    #[must_use]
    pub fn screen_to_world(&self, viewport: Vec2, screen: Vec2) -> Vec2 {
        let offset = screen - viewport / 2.0;
        self.position + offset / self.zoom
    }

    /// Convert a world-space point to screen space.
    #[must_use]
    pub fn world_to_screen(&self, viewport: Vec2, world: Vec2) -> Vec2 {
        viewport / 2.0 + (world - self.position) * self.zoom
    }

    /// Bounding box of the viewport in world space.
    #[must_use]
    pub fn viewport_world_rect(&self, viewport: Vec2) -> Rect {
        let half = viewport / (2.0 * self.zoom);
        Rect::from_min_max(self.position - half, self.position + half)
    }

    /// Apply a pan (translation) in screen-space pixels.
    pub fn pan_screen(&mut self, delta_screen: Vec2) {
        self.position -= delta_screen / self.zoom;
    }

    /// Zoom by a multiplicative factor, keeping the world point under
    /// `cursor_screen` fixed on screen.
    ///
    /// This is the "smart zoom" behavior expected from infinite-canvas apps :
    /// zooming with the mouse cursor on a specific element keeps that
    /// element under the cursor.
    pub fn zoom_at(&mut self, factor: f32, viewport: Vec2, cursor_screen: Vec2) {
        let before = self.screen_to_world(viewport, cursor_screen);
        self.zoom = clamp_zoom(self.zoom * factor);
        let after = self.screen_to_world(viewport, cursor_screen);
        self.position += before - after;
    }
}

/// Clamp a zoom factor to `[MIN_ZOOM, MAX_ZOOM]`.
#[must_use]
pub fn clamp_zoom(z: f32) -> f32 {
    z.clamp(MIN_ZOOM, MAX_ZOOM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_to_world_identity() {
        let cam = Camera::IDENTITY;
        let viewport = Vec2::new(800.0, 600.0);
        // Center of viewport maps to camera position.
        assert_eq!(cam.screen_to_world(viewport, viewport / 2.0), Vec2::ZERO);
    }

    #[test]
    fn world_screen_roundtrip() {
        let cam = Camera::new(Vec2::new(100.0, -50.0), 2.0);
        let viewport = Vec2::new(800.0, 600.0);
        let world = Vec2::new(123.0, 456.0);
        let screen = cam.world_to_screen(viewport, world);
        let back = cam.screen_to_world(viewport, screen);
        assert!((back - world).length() < 1e-3);
    }

    #[test]
    fn zoom_keeps_cursor_anchor() {
        let mut cam = Camera::IDENTITY;
        let viewport = Vec2::new(800.0, 600.0);
        let cursor = Vec2::new(200.0, 150.0);
        let before = cam.screen_to_world(viewport, cursor);
        cam.zoom_at(2.0, viewport, cursor);
        let after = cam.screen_to_world(viewport, cursor);
        assert!((after - before).length() < 1e-3);
    }

    #[test]
    fn zoom_clamped() {
        let mut cam = Camera::IDENTITY;
        cam.zoom_at(1e9, Vec2::ONE, Vec2::ZERO);
        assert!((cam.zoom - MAX_ZOOM).abs() < 1e-3);
    }
}
