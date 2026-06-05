//! Top-level [`eframe::App`] implementation for BSE.
//!
//! The `BseApp` owns the canvas state and orchestrates the layout
//! between the toolbar, the central canvas panel, and the status bar.

use bse_canvas::CanvasState;
use bse_ui::{StatusInfo, status_bar, toolbar};
use eframe::egui;

use crate::APP_INFO;
use crate::canvas;

/// Root application state.
pub struct BseApp {
    canvas: CanvasState,
    fps: f32,
    last_frame: Option<std::time::Instant>,
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
        Self {
            canvas: CanvasState::new(),
            fps: 0.0,
            last_frame: None,
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
}

impl eframe::App for BseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_fps();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar(ui, &mut self.canvas.tool);
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
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            canvas::show(ui, &mut self.canvas);
        });

        ctx.request_repaint();
    }
}
