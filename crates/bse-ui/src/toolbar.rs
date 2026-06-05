//! Top toolbar widget : pick the active tool.

use bse_canvas::ToolKind;
use eframe::egui::{self, Button, RichText};

/// Render the tool selection bar at the top of the window.
pub fn toolbar(ui: &mut egui::Ui, current: &mut ToolKind) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        for kind in TOOLS {
            tool_button(ui, current, *kind);
        }
    });
}

/// Tools displayed in the toolbar, in display order.
const TOOLS: &[ToolKind] = &[
    ToolKind::Select,
    ToolKind::Pen,
    ToolKind::Rectangle,
    ToolKind::Ellipse,
    ToolKind::Line,
    ToolKind::Text,
];

fn tool_button(ui: &mut egui::Ui, current: &mut ToolKind, kind: ToolKind) {
    let selected = *current == kind;
    let label = RichText::new(kind.label()).strong();
    let button = if selected {
        Button::new(label).fill(egui::Color32::from_rgb(0xFF, 0xD0, 0x2F))
    } else {
        Button::new(label)
    };
    if ui.add(button).clicked() {
        *current = kind;
    }
}
