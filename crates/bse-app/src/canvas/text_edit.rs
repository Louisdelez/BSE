//! Inline text-edit overlay for `Text` elements.
//!
//! When [`bse_canvas::ToolState::EditingText`] is active, this module
//! paints a floating egui `TextEdit` anchored on the element's screen
//! position. Pressing **Enter** commits the buffer ; pressing
//! **Escape** discards changes ; clicking outside the overlay commits.

use bse_canvas::{CanvasState, ToolState};
use bse_crdt::{CrdtBackend, YrsBackend};
use bse_model::{Element, ElementKind};
use bse_types::Vec2 as WorldVec2;
use eframe::egui::{self, Rect};
use tracing::warn;

/// Render the inline editor if the canvas is currently editing text.
/// Mutates the CRDT document on commit.
pub fn show_overlay(
    ctx: &egui::Context,
    panel_rect: Rect,
    canvas: &mut CanvasState,
    crdt: &mut YrsBackend,
) {
    let ToolState::EditingText {
        element_id,
        anchor_world,
        buffer,
    } = &mut canvas.tool_state
    else {
        return;
    };

    let viewport = WorldVec2::new(panel_rect.width(), panel_rect.height());
    let screen = canvas.camera.world_to_screen(viewport, *anchor_world);
    let pos = egui::pos2(panel_rect.min.x + screen.x, panel_rect.min.y + screen.y);

    let mut commit = false;
    let mut cancel = false;

    egui::Area::new(egui::Id::new(("bse-text-edit", element_id.as_uuid())))
        .order(egui::Order::Foreground)
        .fixed_pos(pos - egui::vec2(60.0, 12.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                let edit = egui::TextEdit::singleline(buffer)
                    .desired_width(220.0)
                    .hint_text("Type and press Enter");
                let response = ui.add(edit);
                response.request_focus();
                if response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    commit = true;
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel = true;
                }
            });
        });

    if commit {
        let id = *element_id;
        let new_text = std::mem::take(buffer);
        if let Some(mut element) = crdt.get_element(id) {
            apply_text(&mut element, new_text);
            if let Err(err) = crdt.upsert_element(element) {
                warn!(target: "bse::canvas", error = %err, "text edit commit failed");
            }
        }
        canvas.tool_state = ToolState::Idle;
    } else if cancel {
        canvas.tool_state = ToolState::Idle;
    }
}

fn apply_text(element: &mut Element, new_content: String) {
    if let ElementKind::Text { content, .. } = &mut element.kind {
        *content = if new_content.is_empty() {
            "Text".to_string()
        } else {
            new_content
        };
    }
}
