//! Floating tool bar (tldraw-style) — anchored centered-bottom of the
//! window, icon-only with hover tooltips, animated tool select.
//!
//! Icons come from Phosphor via `egui-phosphor`. The active tool gets
//! a brand-yellow background and a subtle scale-up on press.

use bse_canvas::{CanvasState, ToolKind};
use eframe::egui::{self, Color32, Frame, Id, Margin, Rounding, Sense, Shadow, Stroke, Vec2};

use crate::theme::{colors, motion};

/// Tools displayed in the toolbar, in display order.
const TOOLS: &[ToolKind] = &[
    ToolKind::Select,
    ToolKind::Pen,
    ToolKind::Rectangle,
    ToolKind::Ellipse,
    ToolKind::Line,
    ToolKind::Text,
];

/// Render the floating tool bar at the bottom of the parent UI.
///
/// Use it from inside a `CentralPanel` (or any rect-bounded UI). The
/// bar paints over the canvas as a pill-shaped overlay.
pub fn toolbar(ui: &mut egui::Ui, canvas: &mut CanvasState) {
    let parent_rect = ui.max_rect();
    let bar_height = 56.0;
    let bottom_margin = 24.0;

    // Compute the bar size up-front so it stays centered horizontally.
    let count = u32::try_from(TOOLS.len()).unwrap_or(0);
    let icon_size = 44.0;
    let gap = 6.0;
    #[allow(clippy::cast_precision_loss)]
    let bar_width =
        f32::from(u16::try_from(count).unwrap_or(0)).mul_add(icon_size + gap, -gap) + 24.0; // + inner padding

    let center_x = parent_rect.center().x;
    let bottom_y = parent_rect.max.y - bottom_margin - bar_height;
    let bar_rect = egui::Rect::from_min_size(
        egui::Pos2::new(center_x - bar_width / 2.0, bottom_y),
        Vec2::new(bar_width, bar_height),
    );

    let painter = ui.painter();
    // Soft shadow underneath.
    painter.rect(
        bar_rect,
        Rounding::same(bar_height / 2.0),
        Color32::WHITE,
        Stroke::new(1.0, colors::HAIRLINE),
    );
    let shadow = Shadow {
        offset: Vec2::new(0.0, 6.0),
        blur: 24.0,
        spread: -4.0,
        color: Color32::from_black_alpha(30),
    };
    let shadow_shape = shadow.as_shape(bar_rect, Rounding::same(bar_height / 2.0));
    painter.add(shadow_shape);
    // Re-draw the bar on top of its own shadow so the fill stays opaque.
    painter.rect(
        bar_rect,
        Rounding::same(bar_height / 2.0),
        Color32::WHITE,
        Stroke::new(1.0, colors::HAIRLINE),
    );

    let child_builder = egui::UiBuilder::new()
        .max_rect(bar_rect)
        .layout(egui::Layout::left_to_right(egui::Align::Center));
    let mut child = ui.new_child(child_builder);
    Frame::default()
        .inner_margin(Margin::symmetric(12.0, 6.0))
        .show(&mut child, |ui| {
            ui.horizontal_centered(|ui| {
                for (i, kind) in TOOLS.iter().enumerate() {
                    if i > 0 {
                        ui.add_space(gap);
                    }
                    tool_button(ui, canvas, *kind, icon_size);
                }
            });
        });
}

fn tool_button(ui: &mut egui::Ui, canvas: &mut CanvasState, kind: ToolKind, size: f32) {
    let selected = canvas.tool == kind;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());

    // Hover + press animations.
    let id = Id::new(("tool_button", kind));
    let hover_t = ui.ctx().animate_value_with_time(
        id.with("hover"),
        f32::from(u8::from(response.hovered())),
        motion::DURATION_MICRO.as_secs_f32(),
    );
    let press_t = ui.ctx().animate_value_with_time(
        id.with("press"),
        f32::from(u8::from(response.is_pointer_button_down_on())),
        motion::DURATION_MICRO.as_secs_f32() / 2.0,
    );
    let select_t = ui.ctx().animate_value_with_time(
        id.with("select"),
        f32::from(u8::from(selected)),
        motion::DURATION_STANDARD.as_secs_f32(),
    );

    // Scale = 1.0 + a tiny press dip + a small "selected" bump.
    let scale = 1.0 - press_t * 0.04 + select_t * 0.04;
    let target_size = size * scale;
    let scaled_rect = egui::Rect::from_center_size(rect.center(), Vec2::splat(target_size));

    let bg_color = if selected {
        colors::BRAND_YELLOW
    } else if hover_t > 0.0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let alpha = (hover_t * 255.0).clamp(0.0, 255.0) as u8;
        Color32::from_rgba_unmultiplied(
            colors::SURFACE.r(),
            colors::SURFACE.g(),
            colors::SURFACE.b(),
            alpha,
        )
    } else {
        Color32::TRANSPARENT
    };
    let fg_color = if selected { colors::INK } else { colors::SLATE };

    if ui.is_rect_visible(scaled_rect) {
        let painter = ui.painter();
        painter.rect(
            scaled_rect,
            Rounding::same(target_size / 2.0),
            bg_color,
            Stroke::NONE,
        );
        // Icon glyph (Phosphor) centered in the button.
        let glyph = phosphor_for(kind);
        let font_size = target_size * 0.45;
        painter.text(
            scaled_rect.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            egui::FontId::new(font_size, egui::FontFamily::Proportional),
            fg_color,
        );
    }

    response
        .on_hover_text_at_pointer(kind.label())
        .clicked()
        .then(|| canvas.set_tool(kind));
}

fn phosphor_for(tool: ToolKind) -> &'static str {
    use egui_phosphor::regular as ic;
    match tool {
        ToolKind::Select => ic::CURSOR,
        ToolKind::Pen => ic::PENCIL_SIMPLE,
        ToolKind::Rectangle => ic::SQUARE,
        ToolKind::Ellipse => ic::CIRCLE,
        ToolKind::Line => ic::LINE_SEGMENT,
        ToolKind::Text => ic::TEXT_T,
    }
}
