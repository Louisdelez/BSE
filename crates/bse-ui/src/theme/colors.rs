//! Color tokens ported from `/DESIGN.md` (Miro-inspired palette).
//!
//! Every color used in BSE chrome should come from this module. Do
//! not introduce raw `Color32::from_rgb(...)` calls in feature code ;
//! add the value here and reference the constant.
//!
//! Values are kept as `Color32` for direct egui use. Hex notation is
//! preserved in the doc comments for quick correlation with
//! `/DESIGN.md`.

use eframe::egui::Color32;

// ─── Brand ───────────────────────────────────────────────────────────────────

/// `#ffd02f` — Miro canary yellow. Reserved for brand surfaces, the
/// active tool, and selection accents. **Never** use as a primary CTA
/// background.
pub const BRAND_YELLOW: Color32 = Color32::from_rgb(0xFF, 0xD0, 0x2F);
/// `#fcb900` — Pressed / hover state for [`BRAND_YELLOW`].
pub const BRAND_YELLOW_DEEP: Color32 = Color32::from_rgb(0xFC, 0xB9, 0x00);
/// `#fff4c4` — Pale yellow tint for backgrounds.
pub const YELLOW_LIGHT: Color32 = Color32::from_rgb(0xFF, 0xF4, 0xC4);
/// `#746019` — Dark olive used as yellow-tag foreground.
pub const YELLOW_DARK: Color32 = Color32::from_rgb(0x74, 0x60, 0x19);

/// `#4262ff` — Brand blue. Inline links + featured-pricing border.
pub const BRAND_BLUE: Color32 = Color32::from_rgb(0x42, 0x62, 0xFF);
/// `#5b76fe` — Lighter blue, hover state.
pub const BLUE_450: Color32 = Color32::from_rgb(0x5B, 0x76, 0xFE);
/// `#2a41b6` — Pressed-state blue.
pub const BLUE_PRESSED: Color32 = Color32::from_rgb(0x2A, 0x41, 0xB6);

// ─── Pastel accents (post-it palette) ────────────────────────────────────────

/// `#ff9999` — Coral, warm callouts.
pub const BRAND_CORAL: Color32 = Color32::from_rgb(0xFF, 0x99, 0x99);
/// `#ffc6c6` — Pale coral for feature card backgrounds.
pub const CORAL_LIGHT: Color32 = Color32::from_rgb(0xFF, 0xC6, 0xC6);
/// `#600000` — Deep wine, coral-tag foreground.
pub const CORAL_DARK: Color32 = Color32::from_rgb(0x60, 0x00, 0x00);

/// `#ffd8f4` — Brand rose for soft callouts.
pub const BRAND_ROSE: Color32 = Color32::from_rgb(0xFF, 0xD8, 0xF4);
/// `#fde0f0` — Pale rose for feature card backgrounds.
pub const ROSE_LIGHT: Color32 = Color32::from_rgb(0xFD, 0xE0, 0xF0);

/// `#0fbcb0` — Brand teal.
pub const BRAND_TEAL: Color32 = Color32::from_rgb(0x0F, 0xBC, 0xB0);
/// `#c3faf5` — Pale teal for feature card backgrounds.
pub const TEAL_LIGHT: Color32 = Color32::from_rgb(0xC3, 0xFA, 0xF5);
/// `#187574` — Deep teal-green text color.
pub const MOSS_DARK: Color32 = Color32::from_rgb(0x18, 0x75, 0x74);

/// `#ffe6cd` — Soft orange for feature card backgrounds.
pub const ORANGE_LIGHT: Color32 = Color32::from_rgb(0xFF, 0xE6, 0xCD);

// ─── Surfaces ────────────────────────────────────────────────────────────────

/// `#ffffff` — Canvas / page background.
pub const CANVAS: Color32 = Color32::WHITE;
/// `#f7f8fa` — Section backgrounds, search-pill rest.
pub const SURFACE: Color32 = Color32::from_rgb(0xF7, 0xF8, 0xFA);
/// `#fafbfc` — Quieter section divisions.
pub const SURFACE_SOFT: Color32 = Color32::from_rgb(0xFA, 0xFB, 0xFC);
/// `#fff8e0` — Pale yellow-tinted surface for tag chip.
pub const SURFACE_YELLOW: Color32 = Color32::from_rgb(0xFF, 0xF8, 0xE0);
/// `#f5f3ff` — Pale lavender for featured pricing tier.
pub const SURFACE_PRICING_FEATURED: Color32 = Color32::from_rgb(0xF5, 0xF3, 0xFF);

// ─── Borders / hairlines ─────────────────────────────────────────────────────

/// `#e0e2e8` — 1px borders and primary dividers.
pub const HAIRLINE: Color32 = Color32::from_rgb(0xE0, 0xE2, 0xE8);
/// `#eef0f3` — Quieter table-row dividers.
pub const HAIRLINE_SOFT: Color32 = Color32::from_rgb(0xEE, 0xF0, 0xF3);
/// `#c7cad5` — Stronger 1px border for inputs.
pub const HAIRLINE_STRONG: Color32 = Color32::from_rgb(0xC7, 0xCA, 0xD5);

// ─── Text / ink ──────────────────────────────────────────────────────────────

/// `#050038` — Headlines on lighter feature cards.
pub const INK_DEEP: Color32 = Color32::from_rgb(0x05, 0x00, 0x38);
/// `#1c1c1e` — Primary headlines and body text. Also primary CTA bg.
pub const INK: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
/// `#2c2c34` — Body emphasis text. Pressed primary CTA bg.
pub const CHARCOAL: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x34);
/// `#555a6a` — Secondary text, metadata.
pub const SLATE: Color32 = Color32::from_rgb(0x55, 0x5A, 0x6A);
/// `#6b6f7e` — Tertiary text, footer links.
pub const STEEL: Color32 = Color32::from_rgb(0x6B, 0x6F, 0x7E);
/// `#8e91a0` — Captions, muted labels.
pub const STONE: Color32 = Color32::from_rgb(0x8E, 0x91, 0xA0);
/// `#a5a8b5` — Disabled labels, input placeholders.
pub const MUTED: Color32 = Color32::from_rgb(0xA5, 0xA8, 0xB5);

// ─── On-dark text ────────────────────────────────────────────────────────────

/// `#ffffff` — White text on dark surfaces.
pub const ON_DARK: Color32 = Color32::WHITE;
/// `#a5a8b5` — Reduced-opacity white on dark.
pub const ON_DARK_MUTED: Color32 = Color32::from_rgb(0xA5, 0xA8, 0xB5);

// ─── Semantic ────────────────────────────────────────────────────────────────

/// `#00b473` — Success / confirmation green.
pub const SUCCESS: Color32 = Color32::from_rgb(0x00, 0xB4, 0x73);
/// `#fbd4d4` — Soft red for error backgrounds.
pub const ERROR_LIGHT: Color32 = Color32::from_rgb(0xFB, 0xD4, 0xD4);
/// `#e3c5c5` — Stronger red for error borders.
pub const ERROR_BORDER: Color32 = Color32::from_rgb(0xE3, 0xC5, 0xC5);
/// `#b91c1c` — Inline error text (Tailwind red-700 ; clears WCAG AA on CANVAS).
pub const ERROR_TEXT: Color32 = Color32::from_rgb(0xB9, 0x1C, 0x1C);
/// `#e69a00` — Warning / reconnecting indicator.
pub const WARNING: Color32 = Color32::from_rgb(0xE6, 0x9A, 0x00);

// ─── Dark mode overrides ─────────────────────────────────────────────────────

/// Linear-style warm-dark surface — `#0E0E10`.
pub const DARK_CANVAS: Color32 = Color32::from_rgb(0x0E, 0x0E, 0x10);
/// Step up — `#18181B`.
pub const DARK_SURFACE: Color32 = Color32::from_rgb(0x18, 0x18, 0x1B);
/// Step up further — `#27272A`.
pub const DARK_SURFACE_2: Color32 = Color32::from_rgb(0x27, 0x27, 0x2A);
/// Dark hairline `#2E2E33`.
pub const DARK_HAIRLINE: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x33);
/// Slightly desaturated brand yellow for dark mode (avoid glow).
pub const DARK_BRAND_YELLOW: Color32 = Color32::from_rgb(0xD9, 0xB1, 0x28);

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG 2.1 relative luminance (linearized sRGB).
    fn luminance(c: Color32) -> f32 {
        fn channel(v: u8) -> f32 {
            let f = f32::from(v) / 255.0;
            if f <= 0.039_28 {
                f / 12.92
            } else {
                ((f + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }

    fn contrast(fg: Color32, bg: Color32) -> f32 {
        let l1 = luminance(fg).max(luminance(bg));
        let l2 = luminance(fg).min(luminance(bg));
        (l1 + 0.05) / (l2 + 0.05)
    }

    fn assert_aa(fg: Color32, bg: Color32, label: &str) {
        let ratio = contrast(fg, bg);
        assert!(
            ratio >= 4.5,
            "{label}: contrast {ratio:.2}:1 is below WCAG AA (4.5:1)"
        );
    }

    /// Pairings used by the chrome must clear WCAG AA (4.5:1).
    #[test]
    fn light_mode_contrast_aa() {
        assert_aa(INK, CANVAS, "INK on CANVAS");
        assert_aa(SLATE, CANVAS, "SLATE on CANVAS");
        assert_aa(INK, BRAND_YELLOW, "INK on BRAND_YELLOW");
        assert_aa(ON_DARK, INK, "ON_DARK on INK (primary CTA)");
        assert_aa(ERROR_TEXT, CANVAS, "ERROR_TEXT on CANVAS");
        assert_aa(MOSS_DARK, TEAL_LIGHT, "MOSS_DARK on TEAL_LIGHT");
    }

    #[test]
    fn dark_mode_contrast_aa() {
        assert_aa(ON_DARK, DARK_CANVAS, "ON_DARK on DARK_CANVAS");
        assert_aa(ON_DARK, DARK_SURFACE, "ON_DARK on DARK_SURFACE");
        assert_aa(INK, DARK_BRAND_YELLOW, "INK on DARK_BRAND_YELLOW");
    }
}
