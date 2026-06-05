//! Reusable UI components for the BSE desktop app.
//!
//! Every widget here takes `egui` types directly and returns nothing,
//! mutating state through `&mut` references. This keeps composition
//! trivial inside [`eframe::App::update`].

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod components;
mod info;
pub mod pen_options;
mod status_bar;
pub mod theme;
mod toolbar;

pub use components::{
    Card, CardVariant, Command, CommandPaletteState, Modal, ModalResponse, PillButton, PillTone,
    PillVariant, StatusPill, avatar, avatar_stack, card_button, command_palette, show_modal,
};
pub use info::AppInfo;
pub use pen_options::{
    ColorSwatch, DEFAULT_PALETTE, DEFAULT_SIZES, PenOptionsSelection, pen_options,
};
pub use status_bar::{ConnectionStatus, StatusInfo, status_bar};
pub use theme::{ThemeMode, apply_bse_theme};
pub use toolbar::toolbar;
