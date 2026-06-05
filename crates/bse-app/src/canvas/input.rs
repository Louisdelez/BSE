//! Translate `egui` input events into camera mutations.
//!
//! Conventions :
//! - **Pan** : middle-mouse drag, OR primary drag while Space is held.
//! - **Zoom** : scroll wheel, zoom anchored at the cursor position.

use bse_canvas::CanvasState;
use bse_types::Vec2 as WorldVec2;
use eframe::egui::{self, Key, PointerButton, Rect, Response};

/// Multiplier applied to a single scroll-wheel notch.
///
/// Tuned so a typical mouse "click" of the wheel feels like ~10 %
/// zoom change, matching Figma / Miro behavior.
const ZOOM_PER_SCROLL_UNIT: f32 = 0.005;

/// Apply input from the current frame to the canvas state.
pub fn apply(
    ctx: &egui::Context,
    response: &Response,
    rect: Rect,
    viewport: WorldVec2,
    canvas: &mut CanvasState,
) {
    handle_pan(ctx, response, canvas);
    handle_zoom(ctx, response, rect, viewport, canvas);
}

fn handle_pan(ctx: &egui::Context, response: &Response, canvas: &mut CanvasState) {
    let space_held = ctx.input(|i| i.key_down(Key::Space));
    let middle_drag = response.dragged_by(PointerButton::Middle);
    let primary_drag_with_space = response.dragged_by(PointerButton::Primary) && space_held;

    if middle_drag || primary_drag_with_space {
        let delta = response.drag_delta();
        canvas.camera.pan_screen(WorldVec2::new(delta.x, delta.y));
    }
}

fn handle_zoom(
    ctx: &egui::Context,
    response: &Response,
    rect: Rect,
    viewport: WorldVec2,
    canvas: &mut CanvasState,
) {
    if !response.hovered() {
        return;
    }
    let scroll_y = ctx.input(|i| i.raw_scroll_delta.y);
    if scroll_y.abs() < f32::EPSILON {
        return;
    }
    let Some(cursor_global) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    let cursor_local = cursor_global - rect.min.to_vec2();
    let cursor = WorldVec2::new(cursor_local.x, cursor_local.y);
    let factor = 1.0 + scroll_y * ZOOM_PER_SCROLL_UNIT;
    canvas.camera.zoom_at(factor, viewport, cursor);
}
