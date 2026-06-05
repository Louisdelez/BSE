//! Round avatar — colored circle with the first letter of the display
//! name. Used for the presence stack in the status bar.

use eframe::egui::{self, Color32, FontId, Sense, Stroke, Ui, Vec2};

/// Render an avatar of `size` pixels for `display_name` with the
/// provided peer color. Returns the egui response so callers can
/// attach a tooltip.
pub fn avatar(ui: &mut Ui, display_name: &str, color: Color32, size: f32) -> egui::Response {
    let initial = display_name
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_ascii_uppercase().to_string());
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.circle_filled(rect.center(), size / 2.0, color);
        painter.circle_stroke(
            rect.center(),
            size / 2.0,
            Stroke::new(2.0, Color32::WHITE),
        );
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            initial,
            FontId::new(size * 0.5, egui::FontFamily::Proportional),
            text_color_for(color),
        );
    }
    response
}

/// Pick a readable text color (black or white) for the given background
/// using WCAG-style luminance.
fn text_color_for(bg: Color32) -> Color32 {
    let luma = 0.2126 * f32::from(bg.r())
        + 0.7152 * f32::from(bg.g())
        + 0.0722 * f32::from(bg.b());
    if luma > 140.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// Render up to `max_visible` avatars side-by-side, overflowing as
/// "+N" at the end. Each entry is `(display_name, color)`. Returns the
/// total response of the cluster.
pub fn avatar_stack(
    ui: &mut Ui,
    peers: &[(String, Color32)],
    size: f32,
    max_visible: usize,
) -> egui::Response {
    ui.horizontal(|ui| {
        let visible = peers.iter().take(max_visible);
        for (i, (name, color)) in visible.enumerate() {
            if i > 0 {
                ui.add_space(-size * 0.25); // overlap
            }
            avatar(ui, name, *color, size).on_hover_text(name);
        }
        if peers.len() > max_visible {
            let extra = peers.len() - max_visible;
            ui.add_space(-size * 0.25);
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                painter.circle_filled(rect.center(), size / 2.0, crate::theme::colors::SURFACE);
                painter.circle_stroke(
                    rect.center(),
                    size / 2.0,
                    Stroke::new(2.0, Color32::WHITE),
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("+{extra}"),
                    FontId::new(size * 0.35, egui::FontFamily::Proportional),
                    crate::theme::colors::SLATE,
                );
            }
        }
    })
    .response
}
