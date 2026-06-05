//! Renderer trait and placeholder implementation.

use bse_model::{Camera, Scene};
use bse_types::Vec2;

use crate::stats::FrameStats;

/// Abstraction over the renderer.
///
/// The trait makes it easy to plug a "null" renderer for unit tests and
/// a real wgpu renderer for the app.
pub trait Renderer {
    /// Render `scene` from `camera` into a viewport of the given size.
    ///
    /// Returns the number of elements that ended up drawn (after culling).
    fn render(&mut self, scene: &Scene, camera: &Camera, viewport: Vec2) -> FrameStats;
}

/// A no-op renderer that simply counts the visible elements.
///
/// Used in unit tests and as a placeholder until the wgpu pipelines
/// land in v003.
#[derive(Debug, Default)]
pub struct NullRenderer {
    /// Total accumulated frame count, for sanity tests.
    pub frames: u64,
}

impl NullRenderer {
    /// Construct a new null renderer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Renderer for NullRenderer {
    fn render(&mut self, scene: &Scene, camera: &Camera, viewport: Vec2) -> FrameStats {
        self.frames += 1;
        let viewport_rect = camera.viewport_world_rect(viewport);
        let visible_count = scene
            .iter()
            .filter(|e| e.aabb().intersects(&viewport_rect))
            .count();
        FrameStats {
            elements_total: u32::try_from(scene.len()).unwrap_or(u32::MAX),
            elements_visible: u32::try_from(visible_count).unwrap_or(u32::MAX),
            draw_calls: 0,
            cpu_prepare_us: 0,
        }
    }
}
