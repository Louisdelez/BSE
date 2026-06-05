//! Adaptive grid renderer.
//!
//! The grid uses a *minor* and *major* spacing, chosen so that the
//! pixel distance between adjacent minor lines stays in a comfortable
//! range (~12-50 px) regardless of the zoom level. Major lines are
//! drawn every 5 minor lines and stand out slightly more.

use bse_model::Camera;
use bse_types::Vec2 as WorldVec2;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};

const MINOR_COLOR: Color32 = Color32::from_rgb(0xEE, 0xF0, 0xF3);
const MAJOR_COLOR: Color32 = Color32::from_rgb(0xE0, 0xE2, 0xE8);
const MAJOR_EVERY: i32 = 5;
const TARGET_MIN_SPACING_PX: f32 = 14.0;

/// Paint the adaptive grid covering `rect`.
///
/// Cast notes : the visible viewport in world space spans at most a few
/// hundred grid cells in either direction, so the float→`i32` truncation
/// below is safe by construction.
#[allow(clippy::cast_possible_truncation)]
pub fn paint(painter: &egui::Painter, rect: Rect, camera: &Camera, viewport: WorldVec2) {
    let spacing_world = pick_spacing(camera.zoom);
    let world_view = camera.viewport_world_rect(viewport);

    let x_start = (world_view.min.x / spacing_world).floor() as i32;
    let x_end = (world_view.max.x / spacing_world).ceil() as i32;
    let y_start = (world_view.min.y / spacing_world).floor() as i32;
    let y_end = (world_view.max.y / spacing_world).ceil() as i32;

    paint_vertical_lines(
        painter,
        rect,
        camera,
        viewport,
        x_start,
        x_end,
        spacing_world,
    );
    paint_horizontal_lines(
        painter,
        rect,
        camera,
        viewport,
        y_start,
        y_end,
        spacing_world,
    );
}

/// Pick the world-space distance between adjacent minor grid lines so
/// that lines on screen stay around `TARGET_MIN_SPACING_PX` apart.
///
/// Snaps to powers of 10 multiplied by 1, 2 or 5 (the "nice numbers"
/// classically used for axes).
fn pick_spacing(zoom: f32) -> f32 {
    let target_world = TARGET_MIN_SPACING_PX / zoom;
    let exp = target_world.log10().floor();
    let base = 10f32.powf(exp);
    let normalized = target_world / base;
    let nice = if normalized < 2.0 {
        2.0
    } else if normalized < 5.0 {
        5.0
    } else {
        10.0
    };
    nice * base
}

/// Cast note : indices are bounded by the visible viewport, so the
/// `i32` → `f32` conversion stays within `f32` mantissa precision.
#[allow(clippy::cast_precision_loss)]
fn paint_vertical_lines(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    x_start: i32,
    x_end: i32,
    spacing_world: f32,
) {
    for i in x_start..=x_end {
        let x_world = i as f32 * spacing_world;
        let screen = camera.world_to_screen(viewport, WorldVec2::new(x_world, 0.0));
        let x_screen = rect.min.x + screen.x;
        let stroke = line_stroke(i);
        painter.line_segment(
            [
                Pos2::new(x_screen, rect.min.y),
                Pos2::new(x_screen, rect.max.y),
            ],
            stroke,
        );
    }
}

/// Same cast note as [`paint_vertical_lines`].
#[allow(clippy::cast_precision_loss)]
fn paint_horizontal_lines(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    y_start: i32,
    y_end: i32,
    spacing_world: f32,
) {
    for i in y_start..=y_end {
        let y_world = i as f32 * spacing_world;
        let screen = camera.world_to_screen(viewport, WorldVec2::new(0.0, y_world));
        let y_screen = rect.min.y + screen.y;
        let stroke = line_stroke(i);
        painter.line_segment(
            [
                Pos2::new(rect.min.x, y_screen),
                Pos2::new(rect.max.x, y_screen),
            ],
            stroke,
        );
    }
}

fn line_stroke(index: i32) -> Stroke {
    if index.rem_euclid(MAJOR_EVERY) == 0 {
        Stroke::new(1.0, MAJOR_COLOR)
    } else {
        Stroke::new(1.0, MINOR_COLOR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_in_nice_set_at_zoom_1() {
        let s = pick_spacing(1.0);
        assert!(
            [20.0, 50.0, 100.0].iter().any(|n| (s - n).abs() < 0.01),
            "got {s}"
        );
    }

    #[test]
    fn spacing_shrinks_when_zooming_in() {
        let zoomed_out = pick_spacing(0.1);
        let zoomed_in = pick_spacing(10.0);
        assert!(
            zoomed_in < zoomed_out,
            "zoomed in : {zoomed_in}, zoomed out : {zoomed_out}"
        );
    }
}
