//! Feature card with Miro-style rounded corners + soft shadow.
//!
//! Cards are containers — they wrap a closure that draws the actual
//! content. Use [`Card::base`] for everyday neutral cards and
//! [`Card::yellow`] / [`Card::coral`] / [`Card::teal`] / [`Card::rose`]
//! for the pastel feature variants from the Miro design system.

use eframe::egui::{
    Color32, Frame, InnerResponse, Margin, Response, Rounding, Shadow, Stroke, Ui, Vec2,
};

use crate::theme::colors;

/// Visual variant of a card.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CardVariant {
    /// White background, hairline border. The default.
    Base,
    /// Brand-yellow background, ink text.
    Yellow,
    /// Pale coral background, ink text.
    Coral,
    /// Pale teal background, ink text.
    Teal,
    /// Pale rose background, ink text.
    Rose,
    /// Pale orange background, ink text.
    Orange,
    /// Black background, white text. The enterprise/dark variant.
    Dark,
}

/// Miro-style feature card.
pub struct Card {
    variant: CardVariant,
    radius: f32,
    padding: f32,
    shadow: bool,
}

impl Card {
    /// White card with hairline border (most common).
    #[must_use]
    pub fn base() -> Self {
        Self {
            variant: CardVariant::Base,
            radius: 16.0,
            padding: 24.0,
            shadow: false,
        }
    }

    /// Feature card (radius 28 from DESIGN.md, larger padding).
    #[must_use]
    pub fn feature() -> Self {
        Self {
            variant: CardVariant::Base,
            radius: 28.0,
            padding: 32.0,
            shadow: true,
        }
    }

    /// Brand yellow variant.
    #[must_use]
    pub fn yellow() -> Self {
        Self::feature().with_variant(CardVariant::Yellow)
    }

    /// Pale coral variant.
    #[must_use]
    pub fn coral() -> Self {
        Self::feature().with_variant(CardVariant::Coral)
    }

    /// Pale teal variant.
    #[must_use]
    pub fn teal() -> Self {
        Self::feature().with_variant(CardVariant::Teal)
    }

    /// Pale rose variant.
    #[must_use]
    pub fn rose() -> Self {
        Self::feature().with_variant(CardVariant::Rose)
    }

    /// Pale orange variant.
    #[must_use]
    pub fn orange() -> Self {
        Self::feature().with_variant(CardVariant::Orange)
    }

    /// Dark CTA banner variant.
    #[must_use]
    pub fn dark() -> Self {
        Self::feature().with_variant(CardVariant::Dark)
    }

    /// Override the corner radius (defaults : 16 base / 28 feature).
    #[must_use]
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Override the inner padding (defaults : 24 base / 32 feature).
    #[must_use]
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Toggle the soft drop shadow. On by default for feature
    /// variants, off for base.
    #[must_use]
    pub fn shadow(mut self, on: bool) -> Self {
        self.shadow = on;
        self
    }

    fn with_variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Render the card and call `add_contents` to fill it.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let (fill, text_color, stroke) = match self.variant {
            CardVariant::Base => (
                colors::CANVAS,
                colors::INK,
                Stroke::new(1.0, colors::HAIRLINE_SOFT),
            ),
            CardVariant::Yellow => (colors::BRAND_YELLOW, colors::INK, Stroke::NONE),
            CardVariant::Coral => (colors::CORAL_LIGHT, colors::INK, Stroke::NONE),
            CardVariant::Teal => (colors::TEAL_LIGHT, colors::INK, Stroke::NONE),
            CardVariant::Rose => (colors::ROSE_LIGHT, colors::INK, Stroke::NONE),
            CardVariant::Orange => (colors::ORANGE_LIGHT, colors::INK, Stroke::NONE),
            CardVariant::Dark => (colors::INK, colors::ON_DARK, Stroke::NONE),
        };

        let shadow = if self.shadow {
            Shadow {
                offset: Vec2::new(0.0, 12.0),
                blur: 32.0,
                spread: -4.0,
                color: Color32::from_black_alpha(20),
            }
        } else {
            Shadow::NONE
        };

        Frame::default()
            .fill(fill)
            .stroke(stroke)
            .rounding(Rounding::same(self.radius))
            .inner_margin(Margin::same(self.padding))
            .shadow(shadow)
            .show(ui, |ui| {
                ui.style_mut().visuals.override_text_color = Some(text_color);
                add_contents(ui)
            })
    }
}

fn clamp_to_u8(v: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let out = v.clamp(0.0, 255.0) as u8;
    out
}

/// Render `inner` as a clickable surface ; the card highlights on hover.
/// Returns the click response from the *whole card* (not from any inner
/// widget) so feature code can do
/// `if card_button(ui, Card::base(), |ui| ...).clicked() { ... }`.
pub fn card_button<R>(ui: &mut Ui, card: Card, inner: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
    let id = ui.next_auto_id();
    ui.skip_ahead_auto_ids(1);
    let hover_t = ui.ctx().animate_value_with_time(
        id,
        f32::from(u8::from(ui.rect_contains_pointer(ui.max_rect()))),
        crate::theme::motion::DURATION_MICRO.as_secs_f32(),
    );

    let result = card.show(ui, |ui| {
        // Subtle hover tint via background overlay (egui can't blur, so
        // we lift the surface by a tint).
        let rect = ui.max_rect();
        if hover_t > 0.0 {
            let alpha = clamp_to_u8(hover_t * 8.0);
            ui.painter()
                .rect_filled(rect, Rounding::same(16.0), Color32::from_white_alpha(alpha));
        }
        inner(ui)
    });

    let response = ui.interact(result.response.rect, id, eframe::egui::Sense::click());
    (response, result.inner)
}
