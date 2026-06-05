//! BSE theme — design tokens, typography, motion.
//!
//! Apply once at startup with [`apply_bse_theme`] ; afterwards every
//! egui widget picks up the Miro-inspired look automatically.

pub mod colors;
pub mod hot_reload;
pub mod motion;
pub mod typography;

use eframe::egui::{
    self, Color32, Context, FontFamily, FontId, Margin, Rounding, Shadow, Stroke, Style,
    TextStyle, Vec2, Visuals,
};

/// Which theme mode the app is currently rendering in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    /// Light mode — Miro-inspired white canvas.
    #[default]
    Light,
    /// Dark mode — Linear-inspired warm dark.
    Dark,
}

/// Apply the BSE theme to `ctx`. Idempotent : safe to call from
/// `eframe::App::update` if you flip modes at runtime, but cheaper to
/// call once at startup.
pub fn apply_bse_theme(ctx: &Context, mode: ThemeMode) {
    typography::install(ctx);

    let mut style = Style {
        visuals: visuals_for(mode),
        ..Style::default()
    };

    // Type scale — wire our tokens to egui's `TextStyle` enum so
    // `ui.label("...")` picks up the right size by default.
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(22.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(13.0, FontFamily::Proportional),
    );

    // Spacing grid — 4px base from DESIGN.md.
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.menu_margin = Margin::same(8.0);
    style.spacing.window_margin = Margin::same(16.0);
    style.spacing.indent = 18.0;
    style.spacing.icon_width = 16.0;
    style.spacing.scroll.bar_width = 8.0;

    // Rounder defaults — pill buttons handled by component wrappers,
    // here we just bump everything to the "lg" radius for a more
    // modern feel.
    style.visuals.menu_rounding = Rounding::same(12.0);
    style.visuals.window_rounding = Rounding::same(16.0);

    ctx.set_style(style);
}

fn visuals_for(mode: ThemeMode) -> Visuals {
    match mode {
        ThemeMode::Light => light_visuals(),
        ThemeMode::Dark => dark_visuals(),
    }
}

fn light_visuals() -> Visuals {
    let mut v = Visuals::light();

    v.override_text_color = Some(colors::INK);
    v.panel_fill = colors::CANVAS;
    v.window_fill = colors::CANVAS;
    v.window_stroke = Stroke::new(1.0, colors::HAIRLINE);
    v.window_shadow = Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
        color: Color32::from_black_alpha(20),
    };
    v.popup_shadow = v.window_shadow;
    v.faint_bg_color = colors::SURFACE_SOFT;
    v.extreme_bg_color = colors::CANVAS;
    v.code_bg_color = colors::SURFACE;
    v.hyperlink_color = colors::BRAND_BLUE;
    v.selection.bg_fill = colors::BRAND_YELLOW;
    v.selection.stroke = Stroke::new(1.0, colors::INK);

    let radius_md = Rounding::same(8.0);
    let radius_lg = Rounding::same(12.0);

    v.widgets.noninteractive.bg_fill = colors::CANVAS;
    v.widgets.noninteractive.weak_bg_fill = colors::SURFACE_SOFT;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors::HAIRLINE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::INK);
    v.widgets.noninteractive.rounding = radius_md;

    v.widgets.inactive.bg_fill = colors::SURFACE;
    v.widgets.inactive.weak_bg_fill = colors::SURFACE;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, colors::HAIRLINE);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::INK);
    v.widgets.inactive.rounding = radius_md;

    v.widgets.hovered.bg_fill = colors::SURFACE;
    v.widgets.hovered.weak_bg_fill = colors::SURFACE_SOFT;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, colors::HAIRLINE_STRONG);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::INK);
    v.widgets.hovered.rounding = radius_md;

    v.widgets.active.bg_fill = colors::INK;
    v.widgets.active.weak_bg_fill = colors::CHARCOAL;
    v.widgets.active.bg_stroke = Stroke::new(1.0, colors::INK);
    v.widgets.active.fg_stroke = Stroke::new(1.0, colors::ON_DARK);
    v.widgets.active.rounding = radius_md;

    v.widgets.open.bg_fill = colors::SURFACE;
    v.widgets.open.weak_bg_fill = colors::SURFACE;
    v.widgets.open.bg_stroke = Stroke::new(1.0, colors::HAIRLINE_STRONG);
    v.widgets.open.fg_stroke = Stroke::new(1.0, colors::INK);
    v.widgets.open.rounding = radius_lg;

    v
}

fn dark_visuals() -> Visuals {
    let mut v = Visuals::dark();

    v.override_text_color = Some(colors::ON_DARK);
    v.panel_fill = colors::DARK_CANVAS;
    v.window_fill = colors::DARK_SURFACE;
    v.window_stroke = Stroke::new(1.0, colors::DARK_HAIRLINE);
    v.window_shadow = Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
        color: Color32::from_black_alpha(80),
    };
    v.popup_shadow = v.window_shadow;
    v.faint_bg_color = colors::DARK_SURFACE_2;
    v.extreme_bg_color = colors::DARK_CANVAS;
    v.code_bg_color = colors::DARK_SURFACE_2;
    v.hyperlink_color = colors::BLUE_450;
    v.selection.bg_fill = colors::DARK_BRAND_YELLOW;
    v.selection.stroke = Stroke::new(1.0, colors::INK);

    let radius_md = Rounding::same(8.0);

    v.widgets.noninteractive.bg_fill = colors::DARK_CANVAS;
    v.widgets.noninteractive.weak_bg_fill = colors::DARK_SURFACE;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors::DARK_HAIRLINE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::ON_DARK);
    v.widgets.noninteractive.rounding = radius_md;

    v.widgets.inactive.bg_fill = colors::DARK_SURFACE;
    v.widgets.inactive.weak_bg_fill = colors::DARK_SURFACE;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, colors::DARK_HAIRLINE);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::ON_DARK);
    v.widgets.inactive.rounding = radius_md;

    v.widgets.hovered.bg_fill = colors::DARK_SURFACE_2;
    v.widgets.hovered.weak_bg_fill = colors::DARK_SURFACE_2;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, colors::DARK_HAIRLINE);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::ON_DARK);
    v.widgets.hovered.rounding = radius_md;

    v.widgets.active.bg_fill = colors::ON_DARK;
    v.widgets.active.weak_bg_fill = colors::ON_DARK_MUTED;
    v.widgets.active.bg_stroke = Stroke::new(1.0, colors::ON_DARK);
    v.widgets.active.fg_stroke = Stroke::new(1.0, colors::INK);
    v.widgets.active.rounding = radius_md;

    v.widgets.open.bg_fill = colors::DARK_SURFACE_2;
    v.widgets.open.weak_bg_fill = colors::DARK_SURFACE_2;
    v.widgets.open.bg_stroke = Stroke::new(1.0, colors::DARK_HAIRLINE);
    v.widgets.open.fg_stroke = Stroke::new(1.0, colors::ON_DARK);
    v.widgets.open.rounding = radius_md;

    v
}
