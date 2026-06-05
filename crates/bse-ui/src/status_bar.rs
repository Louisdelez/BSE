//! Bottom status bar widget — connection pill + presence avatars +
//! compact debug info.

use bse_canvas::ToolKind;
use eframe::egui::{self, RichText};

use crate::components::{PillTone, StatusPill, avatar_stack};
use crate::info::AppInfo;
use crate::theme::{colors, typography};

/// Connection status mirrored from `bse_sync::ConnectionState`.
///
/// Defined locally so `bse-ui` does not depend on the sync crate.
/// The app maps between the two enums.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// No server configured or user not signed in.
    #[default]
    Offline,
    /// Currently attempting to connect.
    Connecting,
    /// Connected and synchronizing.
    Connected,
    /// Lost connection, retrying with exponential backoff.
    Reconnecting,
}

/// Snapshot of the values displayed in the status bar.
#[derive(Clone, Debug)]
pub struct StatusInfo<'a> {
    /// Static app metadata.
    pub app: AppInfo,
    /// Current camera zoom factor (`1.0` is 100%).
    pub zoom: f32,
    /// Current frames per second.
    pub fps: f32,
    /// Number of connected peers (`0` when offline / solo).
    pub peer_count: u32,
    /// Connection state used to render the bottom-right pill.
    pub connection: ConnectionStatus,
    /// Currently active tool.
    pub tool: ToolKind,
    /// Number of elements currently in the scene.
    pub element_count: u32,
    /// Number of elements that were actually drawn last frame
    /// (after viewport culling). Always `≤ element_count`.
    pub visible_count: u32,
    /// Visible peer display names + assigned colors. Renders as an
    /// avatar stack on the right.
    pub peers: &'a [(String, egui::Color32)],
    /// `true` shows developer-facing details (FPS, element count).
    /// Off by default in production builds.
    pub show_debug: bool,
}

/// Render the bottom status bar.
pub fn status_bar(ui: &mut egui::Ui, info: &StatusInfo<'_>) {
    ui.horizontal(|ui| {
        // Left cluster — minimal app identity.
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("{} {}", info.app.name, info.app.version))
                .color(colors::STEEL)
                .font(typography::size::caption()),
        );

        ui.add_space(12.0);
        ui.label(
            RichText::new(format!("{:>4.0}%", info.zoom * 100.0))
                .color(colors::SLATE)
                .font(typography::size::caption()),
        );

        if info.show_debug {
            ui.add_space(12.0);
            ui.label(
                RichText::new(format!(
                    "{} elements ({} drawn) · {:>3.0} FPS · {}",
                    info.element_count,
                    info.visible_count,
                    info.fps,
                    info.tool.label(),
                ))
                .color(colors::MUTED)
                .font(typography::size::caption()),
            );
        }

        // Right cluster — presence avatars + connection pill.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            let (label, tone) = match info.connection {
                ConnectionStatus::Offline => ("Offline", PillTone::Neutral),
                ConnectionStatus::Connecting => ("Connecting…", PillTone::Brand),
                ConnectionStatus::Connected => ("Live", PillTone::Success),
                ConnectionStatus::Reconnecting => ("Reconnecting…", PillTone::Warning),
            };
            StatusPill::new(label, tone).ui(ui);

            if !info.peers.is_empty() {
                ui.add_space(8.0);
                avatar_stack(ui, info.peers, 24.0, 4);
            }
        });
    });
}
