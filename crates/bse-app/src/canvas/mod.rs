//! The central canvas region of the application.
//!
//! The canvas is responsible for :
//! - converting user input into camera mutations (pan, zoom),
//! - painting an adaptive grid background,
//! - painting the world origin marker for orientation.
//!
//! Submodules :
//! - [`panel`] — the entry point composed into the central `egui` panel.
//! - [`input`] — pure functions translating `egui` input into camera ops.
//! - [`grid`] — adaptive grid renderer.

mod grid;
mod input;
mod panel;

pub use panel::show;
