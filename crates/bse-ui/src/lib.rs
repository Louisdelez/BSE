//! Reusable UI components for the BSE desktop app.
//!
//! Every widget here takes `egui` types directly and returns nothing,
//! mutating state through `&mut` references. This keeps composition
//! trivial inside [`eframe::App::update`](eframe::App::update).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod components;
mod info;
mod status_bar;
pub mod theme;
mod toolbar;

pub use components::{
    Card, CardVariant, Modal, ModalResponse, PillButton, PillTone, PillVariant, StatusPill,
    avatar, avatar_stack, card_button, show_modal,
};
pub use info::AppInfo;
pub use status_bar::{ConnectionStatus, StatusInfo, status_bar};
pub use theme::{ThemeMode, apply_bse_theme};
pub use toolbar::toolbar;
