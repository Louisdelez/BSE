//! Miro-style pill button (`{rounded.full}` + brand colors).
//!
//! Three variants ship today :
//!
//! - [`PillButton::primary`] — black pill, the dominant CTA.
//! - [`PillButton::secondary`] — outlined pill for secondary actions.
//! - [`PillButton::ghost`] — transparent, for tertiary actions.
//!
//! Hover is animated via `Context::animate_value_with_time` on a
//! 0-to-1 progress, mapped to the [`crate::theme::motion::DURATION_MICRO`]
//! token.

use eframe::egui::{
    self, Color32, Id, Response, Rounding, Sense, Stroke, TextStyle, Ui, Vec2, Widget, WidgetText,
};

use crate::theme::{colors, motion};

/// Visual variant of the pill button.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PillVariant {
    /// Black background, white text. The dominant CTA.
    Primary,
    /// Brand yellow background, ink text. Brand emphasis.
    Yellow,
    /// Brand blue background, white text.
    Blue,
    /// Transparent background with hairline border, ink text.
    Secondary,
    /// Transparent background, ink text, no border. Tertiary.
    Ghost,
}

/// Miro-style pill button.
pub struct PillButton<'a> {
    text: WidgetText,
    variant: PillVariant,
    enabled: bool,
    min_size: Option<Vec2>,
    id_source: Option<&'a str>,
}

impl<'a> PillButton<'a> {
    /// Black primary CTA — the dominant button.
    pub fn primary(text: impl Into<WidgetText>) -> Self {
        Self::new(text, PillVariant::Primary)
    }

    /// Brand-yellow CTA — reserved for moments of brand emphasis.
    pub fn yellow(text: impl Into<WidgetText>) -> Self {
        Self::new(text, PillVariant::Yellow)
    }

    /// Brand-blue CTA — inline action callouts.
    pub fn blue(text: impl Into<WidgetText>) -> Self {
        Self::new(text, PillVariant::Blue)
    }

    /// Outlined secondary button.
    pub fn secondary(text: impl Into<WidgetText>) -> Self {
        Self::new(text, PillVariant::Secondary)
    }

    /// Transparent tertiary button.
    pub fn ghost(text: impl Into<WidgetText>) -> Self {
        Self::new(text, PillVariant::Ghost)
    }

    fn new(text: impl Into<WidgetText>, variant: PillVariant) -> Self {
        Self {
            text: text.into(),
            variant,
            enabled: true,
            min_size: None,
            id_source: None,
        }
    }

    /// Override the enabled state. Disabled pills don't respond to
    /// hover/click.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Force a minimum size. Useful for aligning a row of pills.
    #[must_use]
    pub fn min_size(mut self, size: Vec2) -> Self {
        self.min_size = Some(size);
        self
    }

    /// Tag the button so the hover animation gets a stable id.
    #[must_use]
    pub fn id_source(mut self, id: &'a str) -> Self {
        self.id_source = Some(id);
        self
    }
}

impl Widget for PillButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            text,
            variant,
            enabled,
            min_size,
            id_source,
        } = self;

        // Compute the layout (label + padding 12×24 from DESIGN.md).
        let padding = Vec2::new(24.0, 12.0);
        let galley = text.into_galley(ui, Some(egui::TextWrapMode::Extend), 0.0, TextStyle::Button);
        let desired_size = (galley.size() + padding * 2.0).max(min_size.unwrap_or(Vec2::ZERO));

        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(desired_size, sense);

        // Animate hover progress 0 → 1 over DURATION_MICRO.
        let id = id_source.map_or(response.id, |s| Id::new(("pill_button", s)));
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let hover_t = ui.ctx().animate_value_with_time(
            id.with("hover"),
            f32::from(u8::from(hovered)),
            motion::DURATION_MICRO.as_secs_f32(),
        );
        let press_t = ui.ctx().animate_value_with_time(
            id.with("press"),
            f32::from(u8::from(pressed)),
            motion::DURATION_MICRO.as_secs_f32() / 2.0,
        );

        if ui.is_rect_visible(rect) {
            let style = pill_style(variant, enabled, hover_t, press_t);
            let painter = ui.painter();
            // Pill = full radius. Use min(height) / 2 so it stays a true
            // pill regardless of width.
            let radius = rect.height() / 2.0;
            let bg_rect = rect;
            painter.rect(bg_rect, Rounding::same(radius), style.fill, style.stroke);
            painter.galley(rect.center() - galley.size() / 2.0, galley, style.text);
        }

        response
    }
}

struct PillStyle {
    fill: Color32,
    stroke: Stroke,
    text: Color32,
}

fn pill_style(variant: PillVariant, enabled: bool, hover_t: f32, press_t: f32) -> PillStyle {
    if !enabled {
        return PillStyle {
            fill: colors::HAIRLINE,
            stroke: Stroke::NONE,
            text: colors::MUTED,
        };
    }

    let lerp = |from: Color32, to: Color32, t: f32| -> Color32 {
        let t = t.clamp(0.0, 1.0);
        let mix = |a: u8, b: u8| -> u8 {
            let v = f32::from(a).mul_add(1.0 - t, f32::from(b) * t);
            // Clamp explicitly so the f32->u8 cast can never overflow.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let out = v.clamp(0.0, 255.0) as u8;
            out
        };
        Color32::from_rgba_unmultiplied(
            mix(from.r(), to.r()),
            mix(from.g(), to.g()),
            mix(from.b(), to.b()),
            mix(from.a(), to.a()),
        )
    };

    match variant {
        PillVariant::Primary => {
            let base = colors::INK;
            let hovered = colors::CHARCOAL;
            let pressed = Color32::from_rgb(0x0A, 0x0A, 0x0C);
            let fill = lerp(lerp(base, hovered, hover_t), pressed, press_t);
            PillStyle {
                fill,
                stroke: Stroke::NONE,
                text: colors::ON_DARK,
            }
        }
        PillVariant::Yellow => {
            let base = colors::BRAND_YELLOW;
            let hovered = colors::BRAND_YELLOW_DEEP;
            let fill = lerp(base, hovered, hover_t.max(press_t));
            PillStyle {
                fill,
                stroke: Stroke::NONE,
                text: colors::INK,
            }
        }
        PillVariant::Blue => {
            let base = colors::BRAND_BLUE;
            let hovered = colors::BLUE_450;
            let pressed = colors::BLUE_PRESSED;
            let fill = lerp(lerp(base, hovered, hover_t), pressed, press_t);
            PillStyle {
                fill,
                stroke: Stroke::NONE,
                text: colors::ON_DARK,
            }
        }
        PillVariant::Secondary => {
            let fill = lerp(Color32::TRANSPARENT, colors::SURFACE, hover_t);
            PillStyle {
                fill,
                stroke: Stroke::new(1.0, colors::HAIRLINE_STRONG),
                text: colors::INK,
            }
        }
        PillVariant::Ghost => {
            let fill = lerp(Color32::TRANSPARENT, colors::SURFACE, hover_t);
            PillStyle {
                fill,
                stroke: Stroke::NONE,
                text: colors::INK,
            }
        }
    }
}
