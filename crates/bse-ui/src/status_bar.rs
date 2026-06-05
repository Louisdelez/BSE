//! Bottom status bar widget.

use bse_canvas::ToolKind;
use eframe::egui::{self, Color32, RichText};

use crate::info::AppInfo;

/// Snapshot of the values displayed in the status bar.
#[derive(Clone, Copy, Debug)]
pub struct StatusInfo {
    /// Static app metadata.
    pub app: AppInfo,
    /// Current camera zoom factor (`1.0` is 100%).
    pub zoom: f32,
    /// Current frames per second.
    pub fps: f32,
    /// Number of connected peers (`0` when offline / solo).
    pub peer_count: u32,
    /// Currently active tool.
    pub tool: ToolKind,
    /// Number of elements currently in the scene.
    pub element_count: u32,
    /// Number of elements that were actually drawn last frame
    /// (after viewport culling). Always `≤ element_count`.
    pub visible_count: u32,
}

/// Render the bottom status bar.
pub fn status_bar(ui: &mut egui::Ui, info: StatusInfo) {
    ui.horizontal(|ui| {
        let muted = Color32::from_rgb(0x6B, 0x6F, 0x7E);
        ui.label(RichText::new(format!("{} {}", info.app.name, info.app.version)).color(muted));
        ui.separator();
        ui.label(info.app.milestone);
        ui.separator();
        ui.label(format!("Tool : {}", info.tool.label()));
        ui.separator();
        ui.label(format!("Zoom : {:>4.0} %", info.zoom * 100.0));
        ui.separator();
        ui.label(format!(
            "Elements : {} ({} visible)",
            info.element_count, info.visible_count
        ));
        ui.separator();
        ui.label(format!("FPS : {:>3.0}", info.fps));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let peers = info.peer_count;
            let icon = if peers == 0 {
                "● Offline"
            } else {
                "● Online"
            };
            let color = if peers == 0 {
                Color32::from_rgb(0xA5, 0xA8, 0xB5)
            } else {
                Color32::from_rgb(0x00, 0xB4, 0x73)
            };
            ui.colored_label(color, icon);
            if peers > 0 {
                ui.label(format!("{peers} peer(s)"));
            }
        });
    });
}
