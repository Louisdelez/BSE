//! Translate `egui` input events into camera and scene mutations.
//!
//! Conventions :
//! - **Pan** : middle-mouse drag, OR primary drag while Space is held.
//! - **Zoom** : scroll wheel, anchored at the cursor position.
//! - **Drag-to-create** : primary drag while a shape tool (Rectangle,
//!   Ellipse, Line) is active and Space is *not* held.

use bse_canvas::{CanvasState, ToolKind, ToolState};
use bse_model::Scene;
use bse_types::Vec2 as WorldVec2;
use eframe::egui::{self, Key, PointerButton, Rect, Response};

use crate::canvas::draw;

/// Multiplier applied to a single scroll-wheel notch. Tuned for ~10 %
/// zoom change per wheel "click", matching Figma / Miro feel.
const ZOOM_PER_SCROLL_UNIT: f32 = 0.005;

/// Apply input from the current frame.
pub fn apply(
    ctx: &egui::Context,
    response: &Response,
    rect: Rect,
    viewport: WorldVec2,
    canvas: &mut CanvasState,
    scene: &mut Scene,
) {
    let space_held = ctx.input(|i| i.key_down(Key::Space));
    handle_pan(response, canvas, space_held);
    handle_zoom(ctx, response, rect, viewport, canvas);
    handle_drawing(ctx, response, rect, viewport, canvas, scene, space_held);
}

fn handle_pan(response: &Response, canvas: &mut CanvasState, space_held: bool) {
    let middle_drag = response.dragged_by(PointerButton::Middle);
    let primary_drag_with_space = response.dragged_by(PointerButton::Primary) && space_held;
    if middle_drag || primary_drag_with_space {
        let delta = response.drag_delta();
        canvas
            .camera
            .pan_screen(WorldVec2::new(delta.x, delta.y));
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

fn handle_drawing(
    ctx: &egui::Context,
    response: &Response,
    rect: Rect,
    viewport: WorldVec2,
    canvas: &mut CanvasState,
    scene: &mut Scene,
    space_held: bool,
) {
    if !matches!(
        canvas.tool,
        ToolKind::Rectangle | ToolKind::Ellipse | ToolKind::Line
    ) {
        return;
    }
    if space_held {
        return; // space-drag is reserved for pan
    }

    if response.drag_started_by(PointerButton::Primary) {
        if let Some(start_world) = cursor_world(ctx, rect, viewport, canvas) {
            canvas.tool_state = ToolState::DrawingShape {
                anchor_world: start_world,
                current_world: start_world,
            };
        }
    } else if response.dragged_by(PointerButton::Primary) {
        if let ToolState::DrawingShape { anchor_world, .. } = canvas.tool_state
            && let Some(world) = cursor_world(ctx, rect, viewport, canvas)
        {
            canvas.tool_state = ToolState::DrawingShape {
                anchor_world,
                current_world: world,
            };
        }
    } else if response.drag_stopped_by(PointerButton::Primary)
        && let ToolState::DrawingShape {
            anchor_world,
            current_world,
        } = canvas.tool_state
    {
        if let Some(element) = draw::commit_shape(canvas.tool, anchor_world, current_world) {
            scene.insert(element);
        }
        canvas.tool_state = ToolState::Idle;
    }
}

fn cursor_world(
    ctx: &egui::Context,
    rect: Rect,
    viewport: WorldVec2,
    canvas: &CanvasState,
) -> Option<WorldVec2> {
    let global = ctx.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()))?;
    let local = global - rect.min.to_vec2();
    let screen = WorldVec2::new(local.x, local.y);
    Some(canvas.camera.screen_to_world(viewport, screen))
}
