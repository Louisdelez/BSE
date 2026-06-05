//! Top toolbar widget : pick the active tool.

use bse_canvas::{CanvasState, ToolKind};
use eframe::egui::{self, Button, RichText};

/// Tools displayed in the toolbar, in display order.
const TOOLS: &[ToolKind] = &[
    ToolKind::Select,
    ToolKind::Pen,
    ToolKind::Rectangle,
    ToolKind::Ellipse,
    ToolKind::Line,
    ToolKind::Text,
];

/// Render the tool selection bar at the top of the window.
///
/// Switching tools through this widget calls
/// [`CanvasState::set_tool`](bse_canvas::CanvasState::set_tool), which
/// also resets any in-progress tool interaction (drag-to-draw, etc.).
pub fn toolbar(ui: &mut egui::Ui, canvas: &mut CanvasState) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        for kind in TOOLS {
            tool_button(ui, canvas, *kind);
        }
    });
}

fn tool_button(ui: &mut egui::Ui, canvas: &mut CanvasState, kind: ToolKind) {
    let selected = canvas.tool == kind;
    let label = RichText::new(kind.label()).strong();
    let button = if selected {
        Button::new(label).fill(egui::Color32::from_rgb(0xFF, 0xD0, 0x2F))
    } else {
        Button::new(label)
    };
    if ui.add(button).clicked() {
        canvas.set_tool(kind);
    }
}
