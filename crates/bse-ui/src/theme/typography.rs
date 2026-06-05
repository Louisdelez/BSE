//! Typography tokens — Inter Variable font + size scale from `/DESIGN.md`.
//!
//! The font is embedded at compile time via `include_bytes!` so the
//! binary is self-contained (no runtime asset lookup needed).

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};

/// Inter 4.0 variable font, ~843 KB. Single file covers weights
/// 100-900 and the optical-size axis 14-32.
const INTER_VARIABLE: &[u8] = include_bytes!("../../assets/fonts/InterVariable.ttf");

/// Logical font names registered in [`FontDefinitions`].
pub const INTER: &str = "inter";

/// Install Inter as the default proportional font and register
/// Phosphor icons in the same `FontDefinitions`.
///
/// Call this once at startup, *before* any UI is rendered.
pub fn install(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    fonts
        .font_data
        .insert(INTER.to_owned(), FontData::from_static(INTER_VARIABLE).into());

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, INTER.to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, INTER.to_owned());

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);
}

/// Type-scale tokens from `/DESIGN.md` (Roobert PRO scale, adapted to
/// Inter). Returned as `FontId` so they're plug-and-play with
/// `RichText::font` and `ui.label_size`.
pub mod size {
    use eframe::egui::{FontFamily, FontId};

    /// 80px / 500 — marketing hero.
    #[must_use]
    pub fn hero_display() -> FontId {
        FontId::new(80.0, FontFamily::Proportional)
    }

    /// 60px / 500 — major section openers.
    #[must_use]
    pub fn display_lg() -> FontId {
        FontId::new(60.0, FontFamily::Proportional)
    }

    /// 48px / 500 — page-level headlines.
    #[must_use]
    pub fn heading_1() -> FontId {
        FontId::new(48.0, FontFamily::Proportional)
    }

    /// 36px / 500 — subsection headlines.
    #[must_use]
    pub fn heading_2() -> FontId {
        FontId::new(36.0, FontFamily::Proportional)
    }

    /// 28px / 500 — card titles.
    #[must_use]
    pub fn heading_3() -> FontId {
        FontId::new(28.0, FontFamily::Proportional)
    }

    /// 22px / 500 — feature tile titles. The most common dialog title size.
    #[must_use]
    pub fn heading_4() -> FontId {
        FontId::new(22.0, FontFamily::Proportional)
    }

    /// 18px / 500 — FAQ questions, smaller cards.
    #[must_use]
    pub fn heading_5() -> FontId {
        FontId::new(18.0, FontFamily::Proportional)
    }

    /// 18px / 400 — hero subtitle.
    #[must_use]
    pub fn subtitle() -> FontId {
        FontId::new(18.0, FontFamily::Proportional)
    }

    /// 16px / 400 — primary body text. The default.
    #[must_use]
    pub fn body_md() -> FontId {
        FontId::new(16.0, FontFamily::Proportional)
    }

    /// 14px / 400 — secondary body, table cells.
    #[must_use]
    pub fn body_sm() -> FontId {
        FontId::new(14.0, FontFamily::Proportional)
    }

    /// 14px / 500 — button labels.
    #[must_use]
    pub fn button() -> FontId {
        FontId::new(14.0, FontFamily::Proportional)
    }

    /// 13px / 400 — helper text.
    #[must_use]
    pub fn caption() -> FontId {
        FontId::new(13.0, FontFamily::Proportional)
    }

    /// 12px / 500 — footer microcopy.
    #[must_use]
    pub fn micro() -> FontId {
        FontId::new(12.0, FontFamily::Proportional)
    }
}
