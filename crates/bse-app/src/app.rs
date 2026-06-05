//! Top-level [`eframe::App`] implementation for BSE.
//!
//! The `BseApp` owns the canvas state, the scene, and the spatial
//! index, and orchestrates the layout between the toolbar, the central
//! canvas panel, and the status bar.

use bse_canvas::CanvasState;
use bse_model::Scene;
use bse_spatial::Quadtree;
use bse_types::{ElementId, Rect as WorldRect, Vec2 as WorldVec2};
use bse_ui::{StatusInfo, status_bar, toolbar};
use eframe::egui;

use crate::APP_INFO;
use crate::canvas;

/// Half-extent in world units of the [`BseApp::spatial`] index bounds.
///
/// Elements past this boundary still render but lose the spatial index
/// acceleration. Increase if the canvas needs to extend further.
const SPATIAL_HALF_EXTENT: f32 = 1_000_000.0;
const SPATIAL_MAX_ITEMS_PER_LEAF: usize = 16;
const SPATIAL_MAX_DEPTH: u32 = 10;

/// Root application state.
pub struct BseApp {
    canvas: CanvasState,
    scene: Scene,
    spatial: Quadtree<ElementId>,
    fps: f32,
    last_frame: Option<std::time::Instant>,
    last_visible_count: u32,
}

impl Default for BseApp {
    fn default() -> Self {
        Self::new()
    }
}

impl BseApp {
    /// Build a fresh app with default state.
    #[must_use]
    pub fn new() -> Self {
        let bounds = WorldRect::from_min_max(
            WorldVec2::splat(-SPATIAL_HALF_EXTENT),
            WorldVec2::splat(SPATIAL_HALF_EXTENT),
        );
        Self {
            canvas: CanvasState::new(),
            scene: Scene::new(),
            spatial: Quadtree::new(bounds, SPATIAL_MAX_ITEMS_PER_LEAF, SPATIAL_MAX_DEPTH),
            fps: 0.0,
            last_frame: None,
            last_visible_count: 0,
        }
    }

    fn update_fps(&mut self) {
        let now = std::time::Instant::now();
        if let Some(prev) = self.last_frame {
            let dt = now.duration_since(prev).as_secs_f32();
            if dt > 0.0 {
                let instant = 1.0 / dt;
                self.fps = self.fps.mul_add(0.9, instant * 0.1);
            }
        }
        self.last_frame = Some(now);
    }

    /// Rebuild the spatial index from the current scene.
    ///
    /// v007 rebuilds the whole index every frame for simplicity. This
    /// is cheap for typical brainstorming-canvas sizes (< 10 000 elements)
    /// and avoids needing a write-through `Scene` API.
    /// Incremental updates land in v007.1.
    fn rebuild_spatial(&mut self) {
        self.spatial.clear();
        for element in self.scene.iter() {
            self.spatial.insert(element.id, element.aabb());
        }
    }
}

impl eframe::App for BseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_fps();
        self.rebuild_spatial();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar(ui, &mut self.canvas);
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            status_bar(
                ui,
                StatusInfo {
                    app: APP_INFO,
                    zoom: self.canvas.camera.zoom,
                    fps: self.fps,
                    peer_count: 0,
                    tool: self.canvas.tool,
                    element_count: u32::try_from(self.scene.len()).unwrap_or(u32::MAX),
                    visible_count: self.last_visible_count,
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.last_visible_count =
                canvas::show(ui, &mut self.canvas, &mut self.scene, &self.spatial);
        });

        ctx.request_repaint();
    }
}
