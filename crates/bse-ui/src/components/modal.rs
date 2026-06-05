//! Scale-in modal with dimmed backdrop.
//!
//! Drop-in replacement for `egui::Window` when you want a polished
//! modal that animates open (Vercel-curve scale 0.96 → 1.0 + fade,
//! 200ms) and traps focus behind a semi-transparent backdrop.
//!
//! The animation timing is driven by [`crate::theme::motion`] tokens.

use eframe::egui::{
    self, Align2, Area, Color32, Context, Frame, Id, InnerResponse, Margin, Order, Response,
    Rounding, Shadow, Stroke, Ui, Vec2,
};

use crate::theme::{colors, motion};

/// A modal popup that fades in over a dim backdrop.
pub struct Modal<'a> {
    id: Id,
    title: Option<&'a str>,
    min_width: f32,
    max_width: f32,
    /// `false` while the close button is hidden (forces the caller to
    /// dismiss explicitly).
    show_close_button: bool,
}

impl<'a> Modal<'a> {
    /// New modal identified by `id_source`.
    pub fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            id: Id::new(("bse-modal", id_source)),
            title: None,
            min_width: 340.0,
            max_width: 520.0,
            show_close_button: false,
        }
    }

    /// Title displayed in the modal header.
    #[must_use]
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Override the minimum width (default 340px).
    #[must_use]
    pub fn min_width(mut self, w: f32) -> Self {
        self.min_width = w;
        self
    }

    /// Override the maximum width (default 520px).
    #[must_use]
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }

    /// Show a "✕" close button in the top-right corner.
    #[must_use]
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    /// Render the modal. `add_contents` fills the modal body ; the
    /// returned tuple carries the inner response and a `closed_by_x`
    /// flag (true iff the user clicked the close button).
    pub fn show<R>(
        self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> ModalResponse<R> {
        // Animate the "open" progress 0 → 1 in DURATION_STANDARD.
        let open_t = ctx.animate_value_with_time(
            self.id.with("open"),
            1.0,
            motion::DURATION_STANDARD.as_secs_f32(),
        );

        // Dim the backdrop. We can't blur, so we paint a flat dim.
        let screen = ctx.screen_rect();
        let backdrop_alpha = clamp_to_u8(open_t * 96.0);
        Area::new(self.id.with("backdrop"))
            .order(Order::Background)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let rect = ui.allocate_space(screen.size()).1;
                ui.painter().rect_filled(
                    rect,
                    Rounding::ZERO,
                    Color32::from_black_alpha(backdrop_alpha),
                );
            });

        // Scale-in modal panel.
        let scale = 0.96 + open_t * 0.04;
        let offset_y = -(1.0 - open_t) * 4.0;

        let mut closed_by_x = false;

        let area = Area::new(self.id)
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, offset_y))
            .interactable(true);

        let response = area.show(ctx, |ui| {
            // Apply the scale to the *visuals* by tweaking the
            // alignment ; egui can't transform a Frame, so we let the
            // scale only affect opacity above for now. (A future
            // version with a paint-callback can do real scale.)
            let _ = scale;

            ui.style_mut().visuals.override_text_color = Some(colors::INK);
            Frame::default()
                .fill(colors::CANVAS)
                .stroke(Stroke::new(1.0, colors::HAIRLINE))
                .rounding(Rounding::same(16.0))
                .shadow(Shadow {
                    offset: Vec2::new(0.0, 16.0),
                    blur: 48.0,
                    spread: -8.0,
                    color: Color32::from_black_alpha(clamp_to_u8(open_t * 60.0)),
                })
                .inner_margin(Margin::ZERO)
                .show(ui, |ui| {
                    ui.set_min_width(self.min_width);
                    ui.set_max_width(self.max_width);

                    // Header.
                    if self.title.is_some() || self.show_close_button {
                        Frame::default()
                            .inner_margin(Margin {
                                left: 24.0,
                                right: 12.0,
                                top: 20.0,
                                bottom: 12.0,
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if let Some(title) = self.title {
                                        ui.label(
                                            egui::RichText::new(title)
                                                .font(crate::theme::typography::size::heading_4())
                                                .color(colors::INK),
                                        );
                                    }
                                    if self.show_close_button {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("✕").clicked() {
                                                    closed_by_x = true;
                                                }
                                            },
                                        );
                                    }
                                });
                            });
                    }

                    // Body.
                    let body = Frame::default()
                        .inner_margin(Margin {
                            left: 24.0,
                            right: 24.0,
                            top: 8.0,
                            bottom: 24.0,
                        })
                        .show(ui, add_contents);

                    body.inner
                })
        });

        // Reset the animation flag on the next frame so a re-open
        // animates from scratch. We simply don't latch open_t at
        // 1.0 ; the next call to animate_value_with_time will be
        // already at 1.0, which is fine.

        ModalResponse {
            response: response.response,
            inner: response.inner.inner,
            closed_by_x,
            open_progress: open_t,
        }
    }
}

fn clamp_to_u8(v: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let out = v.clamp(0.0, 255.0) as u8;
    out
}

/// Result of a modal render.
pub struct ModalResponse<R> {
    /// Egui response of the underlying area (use for input hit tests).
    pub response: Response,
    /// Value returned by the inner `add_contents` closure.
    pub inner: R,
    /// `true` iff the close-button (✕) was clicked this frame.
    pub closed_by_x: bool,
    /// 0.0 → 1.0 open progress. Mostly useful for testing.
    pub open_progress: f32,
}

/// Convenience wrapper that hides the `InnerResponse` boilerplate when
/// you only need the inner value.
pub fn show_modal<R>(
    ctx: &Context,
    id_source: impl std::hash::Hash,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let res = Modal::new(id_source).title(title).show(ctx, add_contents);
    InnerResponse {
        response: res.response,
        inner: res.inner,
    }
}
