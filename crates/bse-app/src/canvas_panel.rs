//! The central `CentralPanel` content : the canvas region.
//!
//! In v003 the canvas is rendered with the egui painter (a uniform
//! colored rectangle). v004 swaps in a `wgpu` paint callback that
//! renders a grid background shader.

use bse_canvas::CanvasState;
use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Sense, Stroke};

/// Render the central canvas region and return the rectangle the canvas occupies.
pub fn show(ui: &mut egui::Ui, _canvas: &mut CanvasState) -> Rect {
    let available = ui.available_size_before_wrap();
    let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());
    let rect = response.rect;

    paint_background(&painter, rect);
    paint_origin_crosshair(&painter, rect);
    paint_placeholder_label(&painter, rect);

    rect
}

fn paint_background(painter: &egui::Painter, rect: Rect) {
    // Miro design system : surface = #F7F8FA (mode clair).
    let surface = Color32::from_rgb(0xF7, 0xF8, 0xFA);
    painter.rect_filled(rect, Rounding::ZERO, surface);
}

fn paint_origin_crosshair(painter: &egui::Painter, rect: Rect) {
    // World origin is at the center of the viewport, since the camera starts
    // at (0, 0) with zoom 1.0. We draw a faint crosshair to confirm.
    let center = rect.center();
    let len = 12.0;
    let stroke = Stroke::new(1.0, Color32::from_rgb(0xC7, 0xCA, 0xD5));
    painter.line_segment(
        [
            Pos2::new(center.x - len, center.y),
            Pos2::new(center.x + len, center.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(center.x, center.y - len),
            Pos2::new(center.x, center.y + len),
        ],
        stroke,
    );
}

fn paint_placeholder_label(painter: &egui::Painter, rect: Rect) {
    let text = "Canvas — v003\nPan / zoom arrive in v004";
    let pos = rect.center() + egui::vec2(0.0, 30.0);
    painter.text(
        pos,
        egui::Align2::CENTER_TOP,
        text,
        egui::FontId::proportional(14.0),
        Color32::from_rgb(0x8E, 0x91, 0xA0),
    );
}
