//! The central canvas widget.

use bse_canvas::CanvasState;
use bse_types::Vec2 as WorldVec2;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke};

use crate::canvas::{grid, input};

/// Render the central canvas region. Returns the screen rectangle it occupies.
pub fn show(ui: &mut egui::Ui, canvas: &mut CanvasState) -> Rect {
    let available = ui.available_size_before_wrap();
    let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());
    let rect = response.rect;
    let viewport_pixels = WorldVec2::new(rect.width(), rect.height());

    input::apply(ui.ctx(), &response, rect, viewport_pixels, canvas);

    paint_background(&painter, rect);
    grid::paint(&painter, rect, &canvas.camera, viewport_pixels);
    paint_origin_marker(&painter, rect, canvas, viewport_pixels);

    rect
}

fn paint_background(painter: &egui::Painter, rect: Rect) {
    let surface = Color32::from_rgb(0xF7, 0xF8, 0xFA);
    painter.rect_filled(rect, 0.0, surface);
}

fn paint_origin_marker(
    painter: &egui::Painter,
    rect: Rect,
    canvas: &CanvasState,
    viewport: WorldVec2,
) {
    let origin_screen = canvas.camera.world_to_screen(viewport, WorldVec2::ZERO);
    let origin = Pos2::new(rect.min.x + origin_screen.x, rect.min.y + origin_screen.y);
    if !rect.expand(20.0).contains(origin) {
        return;
    }
    let len = 10.0;
    let stroke = Stroke::new(1.0, Color32::from_rgb(0xC7, 0xCA, 0xD5));
    painter.line_segment(
        [
            Pos2::new(origin.x - len, origin.y),
            Pos2::new(origin.x + len, origin.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(origin.x, origin.y - len),
            Pos2::new(origin.x, origin.y + len),
        ],
        stroke,
    );
}
