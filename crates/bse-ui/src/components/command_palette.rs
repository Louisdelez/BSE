//! Command palette (Cmd+K) — Figma-style modal search for actions.
//!
//! Commands are registered each frame by the caller. The palette
//! filters them by substring + word-prefix score, lets the user
//! navigate with arrow keys, and fires the selected command on
//! Enter.
//!
//! The widget is stateless from the caller's perspective : feed it
//! the current open flag + commands list, take back a chosen index
//! (or `None`) at the end.

use eframe::egui::{
    self, Align2, Color32, Frame, Id, Key, Margin, Order, Pos2, Rounding, Sense, Shadow, Stroke,
    TextEdit, Ui, Vec2,
};

use crate::theme::{colors, motion, typography};

/// One command exposed in the palette.
#[derive(Clone, Debug)]
pub struct Command {
    /// Stable id (kebab-case). Used as the egui `id_source`.
    pub id: &'static str,
    /// Title shown in the list ("Switch to Pen", "Sign out", …).
    pub title: String,
    /// Optional secondary description shown muted on the right.
    pub hint: Option<String>,
    /// Optional Phosphor glyph (e.g. `egui_phosphor::regular::PENCIL_SIMPLE`).
    pub icon: Option<&'static str>,
    /// Optional category label ("Tool", "Room", …) — surfaced in muted
    /// caption above each group of commands.
    pub category: Option<&'static str>,
}

impl Command {
    /// Convenience constructor for a no-hint, no-icon command.
    pub fn new(id: &'static str, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            hint: None,
            icon: None,
            category: None,
        }
    }

    /// Add a Phosphor glyph displayed left of the title.
    #[must_use]
    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Add a right-side hint (e.g. keyboard shortcut, role).
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Group the command under a category header.
    #[must_use]
    pub fn with_category(mut self, category: &'static str) -> Self {
        self.category = Some(category);
        self
    }
}

/// In-memory state of the palette. Owned by the host app.
#[derive(Default)]
pub struct CommandPaletteState {
    /// `true` when the palette is currently shown.
    pub open: bool,
    /// Current text in the search field.
    pub query: String,
    /// Index of the highlighted command in the filtered list.
    pub highlight: usize,
}

impl CommandPaletteState {
    /// Show the palette and reset its state to a blank search.
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.highlight = 0;
    }

    /// Hide the palette without firing a command.
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.highlight = 0;
    }
}

/// Render the command palette modal. Returns the chosen command's
/// `id` when the user presses Enter (or clicks a row), `None`
/// otherwise.
///
/// `commands` is the *unfiltered* list passed each frame ; this
/// function applies fuzzy matching internally.
#[allow(clippy::too_many_lines)]
pub fn show(
    ctx: &egui::Context,
    state: &mut CommandPaletteState,
    commands: &[Command],
) -> Option<&'static str> {
    if !state.open {
        return None;
    }

    // Esc closes.
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        state.close();
        return None;
    }

    let open_t = ctx.animate_value_with_time(
        Id::new("cmdpalette-open"),
        1.0,
        motion::DURATION_STANDARD.as_secs_f32(),
    );

    // Backdrop.
    let screen = ctx.screen_rect();
    egui::Area::new(Id::new("cmdpalette-backdrop"))
        .order(Order::Background)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let rect = ui.allocate_space(screen.size()).1;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let alpha = (open_t * 96.0).clamp(0.0, 255.0) as u8;
            ui.painter()
                .rect_filled(rect, Rounding::ZERO, Color32::from_black_alpha(alpha));
            if ui
                .interact(rect, Id::new("cmdpalette-backdrop-click"), Sense::click())
                .clicked()
            {
                state.close();
            }
        });

    // Filter + score commands.
    let filtered = filter(commands, &state.query);
    state.highlight = state.highlight.min(filtered.len().saturating_sub(1));

    // Arrow key nav.
    ctx.input(|i| {
        if i.key_pressed(Key::ArrowDown) && !filtered.is_empty() {
            state.highlight = (state.highlight + 1) % filtered.len();
        }
        if i.key_pressed(Key::ArrowUp) && !filtered.is_empty() {
            state.highlight = state.highlight.checked_sub(1).unwrap_or(filtered.len() - 1);
        }
    });

    let enter = ctx.input(|i| i.key_pressed(Key::Enter));

    let mut chosen: Option<&'static str> = None;

    egui::Area::new(Id::new("cmdpalette"))
        .order(Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, -120.0))
        .show(ctx, |ui| {
            Frame::default()
                .fill(colors::CANVAS)
                .stroke(Stroke::new(1.0, colors::HAIRLINE))
                .rounding(Rounding::same(16.0))
                .shadow(Shadow {
                    offset: Vec2::new(0.0, 24.0),
                    blur: 64.0,
                    spread: -10.0,
                    color: Color32::from_black_alpha(50),
                })
                .inner_margin(Margin::ZERO)
                .show(ui, |ui| {
                    ui.set_min_width(560.0);
                    ui.set_max_width(680.0);

                    // Search input row.
                    Frame::default()
                        .inner_margin(Margin {
                            left: 20.0,
                            right: 20.0,
                            top: 16.0,
                            bottom: 8.0,
                        })
                        .show(ui, |ui| {
                            let response = ui.add(
                                TextEdit::singleline(&mut state.query)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("Type a command…")
                                    .font(typography::size::body_md())
                                    .margin(egui::vec2(12.0, 10.0)),
                            );
                            response.request_focus();
                        });

                    ui.separator();

                    // Results.
                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            let mut last_category: Option<&'static str> = None;
                            for (i, cmd) in filtered.iter().enumerate() {
                                if cmd.category != last_category {
                                    if let Some(cat) = cmd.category {
                                        ui.add_space(6.0);
                                        Frame::default()
                                            .inner_margin(Margin {
                                                left: 24.0,
                                                right: 24.0,
                                                top: 6.0,
                                                bottom: 4.0,
                                            })
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(cat.to_uppercase())
                                                        .color(colors::STONE)
                                                        .font(typography::size::micro())
                                                        .strong(),
                                                );
                                            });
                                    }
                                    last_category = cmd.category;
                                }
                                if let Some(id) = render_row(ui, cmd, i == state.highlight, enter) {
                                    chosen = Some(id);
                                }
                            }
                            if filtered.is_empty() {
                                Frame::default()
                                    .inner_margin(Margin::same(24.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(
                                                "No commands match. Press Esc to dismiss.",
                                            )
                                            .color(colors::STEEL)
                                            .font(typography::size::body_sm()),
                                        );
                                    });
                            }
                        });
                });
        });

    if chosen.is_some() {
        state.close();
    }
    chosen
}

#[allow(clippy::too_many_lines)]
fn render_row(ui: &mut Ui, cmd: &Command, highlighted: bool, enter: bool) -> Option<&'static str> {
    let row_height = 44.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), row_height), Sense::click());

    let bg = if highlighted {
        colors::SURFACE
    } else if response.hovered() {
        colors::SURFACE_SOFT
    } else {
        Color32::TRANSPARENT
    };
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect.shrink2(Vec2::new(8.0, 0.0)), Rounding::same(8.0), bg);

        let mut x = rect.min.x + 16.0;

        if let Some(icon) = cmd.icon {
            painter.text(
                Pos2::new(x + 12.0, rect.center().y),
                Align2::CENTER_CENTER,
                icon,
                egui::FontId::new(18.0, egui::FontFamily::Proportional),
                colors::SLATE,
            );
            x += 32.0;
        }

        painter.text(
            Pos2::new(x, rect.center().y),
            Align2::LEFT_CENTER,
            &cmd.title,
            typography::size::body_md(),
            colors::INK,
        );

        if let Some(hint) = cmd.hint.as_ref() {
            painter.text(
                Pos2::new(rect.max.x - 24.0, rect.center().y),
                Align2::RIGHT_CENTER,
                hint,
                typography::size::caption(),
                colors::STEEL,
            );
        }
    }

    if (highlighted && enter) || response.clicked() {
        return Some(cmd.id);
    }
    None
}

/// Filter + score `commands` against `query` using a simple
/// case-insensitive substring + word-prefix score. Empty queries
/// return everything in original order.
fn filter<'a>(commands: &'a [Command], query: &str) -> Vec<&'a Command> {
    if query.trim().is_empty() {
        return commands.iter().collect();
    }
    let q = query.to_ascii_lowercase();
    let mut scored: Vec<(i32, &Command)> = commands
        .iter()
        .filter_map(|c| {
            let title = c.title.to_ascii_lowercase();
            if !title.contains(&q) {
                return None;
            }
            let mut score = 100;
            // Word-prefix match scores higher.
            for word in title.split_whitespace() {
                if word.starts_with(&q) {
                    score += 50;
                    break;
                }
            }
            // Title-start match scores even higher.
            if title.starts_with(&q) {
                score += 100;
            }
            // Shorter titles win ties.
            score -= i32::try_from(title.len()).unwrap_or(i32::MAX);
            Some((score, c))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, c)| c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(id: &'static str, title: &str) -> Command {
        Command::new(id, title)
    }

    #[test]
    fn empty_query_returns_all() {
        let cs = vec![cmd("a", "Foo"), cmd("b", "Bar")];
        assert_eq!(filter(&cs, "").len(), 2);
    }

    #[test]
    fn substring_match() {
        let cs = vec![cmd("a", "Sign out"), cmd("b", "Switch room")];
        let out = filter(&cs, "sign");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "a");
    }

    #[test]
    fn title_start_outranks_substring() {
        let cs = vec![cmd("a", "Open settings"), cmd("b", "Settings export")];
        let out = filter(&cs, "set");
        assert_eq!(out[0].id, "b", "title-start match should rank first");
    }
}
