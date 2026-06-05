//! Connection / role badges — colored pills with a leading dot.
//!
//! Used in the status bar and (later) the room picker to surface
//! state at a glance.

use eframe::egui::{self, Color32, Rounding, Sense, Stroke, Ui, Vec2, WidgetText};

use crate::theme::{colors, typography};

/// Visual tone of the pill.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PillTone {
    /// Subtle / inactive.
    Neutral,
    /// Live / OK.
    Success,
    /// Warning / pending.
    Warning,
    /// Error.
    Error,
    /// Brand accent.
    Brand,
}

impl PillTone {
    fn colors(self) -> (Color32, Color32) {
        match self {
            Self::Neutral => (
                Color32::from_rgba_unmultiplied(colors::MUTED.r(), colors::MUTED.g(), colors::MUTED.b(), 30),
                colors::STEEL,
            ),
            Self::Success => (
                Color32::from_rgba_unmultiplied(0x00, 0xB4, 0x73, 40),
                Color32::from_rgb(0x00, 0x7A, 0x4D),
            ),
            Self::Warning => (
                Color32::from_rgba_unmultiplied(0xE6, 0x9A, 0x00, 50),
                Color32::from_rgb(0x94, 0x65, 0x00),
            ),
            Self::Error => (
                Color32::from_rgba_unmultiplied(0xE6, 0x3A, 0x46, 40),
                colors::ERROR_TEXT,
            ),
            Self::Brand => (colors::SURFACE_YELLOW, colors::YELLOW_DARK),
        }
    }
}

/// A small colored pill with an optional leading dot.
pub struct StatusPill {
    text: WidgetText,
    tone: PillTone,
    dot: bool,
}

impl StatusPill {
    /// New pill with the given label and tone.
    pub fn new(text: impl Into<WidgetText>, tone: PillTone) -> Self {
        Self {
            text: text.into(),
            tone,
            dot: true,
        }
    }

    /// Hide the leading colored dot.
    #[must_use]
    pub fn no_dot(mut self) -> Self {
        self.dot = false;
        self
    }

    /// Render into `ui` and return the response.
    pub fn ui(self, ui: &mut Ui) -> egui::Response {
        let (bg, fg) = self.tone.colors();
        let font = typography::size::caption();
        let galley = self.text.into_galley(ui, Some(egui::TextWrapMode::Extend), 0.0, font);

        let dot_w = if self.dot { 8.0 } else { 0.0 };
        let padding = Vec2::new(10.0, 4.0);
        let desired_size = Vec2::new(
            galley.size().x + dot_w + padding.x * 2.0 + if self.dot { 6.0 } else { 0.0 },
            galley.size().y.max(20.0) + padding.y * 2.0,
        );
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect(
                rect,
                Rounding::same(rect.height() / 2.0),
                bg,
                Stroke::NONE,
            );

            let mut text_left = rect.min.x + padding.x;
            if self.dot {
                let dot_center = egui::Pos2::new(text_left + 4.0, rect.center().y);
                painter.circle_filled(dot_center, 4.0, fg);
                text_left += dot_w + 6.0;
            }
            painter.galley(
                egui::Pos2::new(text_left, rect.center().y - galley.size().y / 2.0),
                galley,
                fg,
            );
        }

        response
    }
}
