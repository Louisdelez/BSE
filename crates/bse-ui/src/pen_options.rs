//! Pen options panel — floating pill that lets the user pick a stroke
//! color and a stroke size, shown above the floating tool bar when
//! the Pen tool is selected.

use eframe::egui::{
    self, Color32, Frame, Margin, Pos2, Rect, Rounding, Sense, Shadow, Stroke, Vec2,
};

use crate::theme::colors;

/// One swatch in the palette. Bytes are world-space `Color` channels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColorSwatch {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
}

impl ColorSwatch {
    /// Construct a swatch.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// As an `egui::Color32`.
    #[must_use]
    pub fn to_egui(self) -> Color32 {
        Color32::from_rgb(self.r, self.g, self.b)
    }
}

/// Default 8-swatch Miro-inspired palette : black, coral, orange,
/// brand yellow, teal, brand blue, purple, rose.
pub const DEFAULT_PALETTE: &[ColorSwatch] = &[
    ColorSwatch::new(0x1C, 0x1C, 0x1E), // ink
    ColorSwatch::new(0xE6, 0x3A, 0x46), // red
    ColorSwatch::new(0xF2, 0x82, 0x14), // orange
    ColorSwatch::new(0xFF, 0xD0, 0x2F), // brand yellow
    ColorSwatch::new(0x0F, 0xBC, 0xB0), // teal
    ColorSwatch::new(0x42, 0x62, 0xFF), // brand blue
    ColorSwatch::new(0x9C, 0x4C, 0xDD), // purple
    ColorSwatch::new(0xFF, 0x99, 0xC8), // rose
];

/// Default size scale, in world units. 5 steps from very thin to bold.
pub const DEFAULT_SIZES: &[f32] = &[2.0, 4.0, 8.0, 16.0, 24.0];

/// Current pen options bound for editing.
#[derive(Copy, Clone, Debug)]
pub struct PenOptionsSelection {
    /// Currently picked color.
    pub color: ColorSwatch,
    /// Currently picked size, world units.
    pub size: f32,
}

/// Render the floating pen-options panel above the tool bar.
///
/// `panel_rect.center_top` will be used as the panel anchor — pass the
/// rect of the bottom-anchored tool bar and the panel will float
/// immediately above it.
pub fn pen_options(
    ui: &mut egui::Ui,
    panel_anchor: Pos2,
    selection: &mut PenOptionsSelection,
    palette: &[ColorSwatch],
    sizes: &[f32],
) {
    let swatch_size = 24.0;
    let size_dot_max = 22.0;
    let inner_gap = 8.0;
    let group_gap = 16.0;
    let padding = Vec2::new(16.0, 10.0);

    let palette_w = f32::from(u16::try_from(palette.len()).unwrap_or(0))
        .mul_add(swatch_size + inner_gap, -inner_gap);
    let sizes_w = f32::from(u16::try_from(sizes.len()).unwrap_or(0))
        .mul_add(size_dot_max + inner_gap, -inner_gap);
    let bar_w = padding.x * 2.0 + palette_w + group_gap + 1.0 + group_gap + sizes_w;
    let bar_h = padding.y * 2.0 + swatch_size.max(size_dot_max);

    // Anchor : bottom of the panel sits 12px above the toolbar's top.
    let center_x = panel_anchor.x;
    let bottom_y = panel_anchor.y - 12.0;
    let bar_rect = Rect::from_min_size(
        Pos2::new(center_x - bar_w / 2.0, bottom_y - bar_h),
        Vec2::new(bar_w, bar_h),
    );

    // Soft shadow underneath.
    let shadow = Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 16.0,
        spread: -2.0,
        color: Color32::from_black_alpha(30),
    };
    ui.painter()
        .add(shadow.as_shape(bar_rect, Rounding::same(bar_h / 2.0)));
    ui.painter().rect(
        bar_rect,
        Rounding::same(bar_h / 2.0),
        Color32::WHITE,
        Stroke::new(1.0, colors::HAIRLINE),
    );

    let child_builder = egui::UiBuilder::new()
        .max_rect(bar_rect)
        .layout(egui::Layout::left_to_right(egui::Align::Center));
    let mut child = ui.new_child(child_builder);
    Frame::default()
        .inner_margin(Margin::symmetric(padding.x, padding.y))
        .show(&mut child, |ui| {
            ui.horizontal_centered(|ui| {
                for (i, swatch) in palette.iter().enumerate() {
                    if i > 0 {
                        ui.add_space(inner_gap);
                    }
                    if color_swatch(ui, *swatch, selection.color == *swatch, swatch_size) {
                        selection.color = *swatch;
                    }
                }

                ui.add_space(group_gap);
                let sep_rect = ui
                    .allocate_exact_size(Vec2::new(1.0, swatch_size), Sense::hover())
                    .0;
                ui.painter()
                    .rect_filled(sep_rect, Rounding::ZERO, colors::HAIRLINE);
                ui.add_space(group_gap);

                for (i, size) in sizes.iter().enumerate() {
                    if i > 0 {
                        ui.add_space(inner_gap);
                    }
                    if size_dot(
                        ui,
                        *size,
                        selection.size,
                        size_dot_max,
                        sizes_normalized_radius(*size, sizes),
                    ) {
                        selection.size = *size;
                    }
                }
            });
        });
}

fn color_swatch(ui: &mut egui::Ui, swatch: ColorSwatch, selected: bool, size: f32) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let inner_color = swatch.to_egui();
        if selected {
            // Outer ring : brand yellow.
            painter.circle_filled(rect.center(), size / 2.0, colors::BRAND_YELLOW);
            painter.circle_filled(rect.center(), size / 2.0 - 3.0, inner_color);
        } else {
            painter.circle_filled(rect.center(), size / 2.0, inner_color);
            painter.circle_stroke(
                rect.center(),
                size / 2.0,
                Stroke::new(1.0, Color32::from_black_alpha(20)),
            );
        }
    }
    response.on_hover_text("Pen color").clicked()
}

fn size_dot(ui: &mut egui::Ui, size: f32, current: f32, button_size: f32, dot_radius: f32) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(button_size), Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let selected = (size - current).abs() < 0.01;
        let bg = if selected {
            colors::SURFACE
        } else {
            Color32::TRANSPARENT
        };
        painter.circle_filled(rect.center(), button_size / 2.0, bg);
        painter.circle_filled(rect.center(), dot_radius, colors::INK);
    }
    response.on_hover_text(format!("Size {size:.0}")).clicked()
}

fn sizes_normalized_radius(size: f32, sizes: &[f32]) -> f32 {
    let max = sizes.iter().fold(0.0_f32, |a, b| a.max(*b));
    if max <= 0.0 {
        return 2.0;
    }
    // Map the smallest 2.0 → radius 2.0, the largest → radius 9.0.
    let t = (size / max).clamp(0.0, 1.0);
    2.0_f32.mul_add(1.0 - t, 9.0 * t)
}
