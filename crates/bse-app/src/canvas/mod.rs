//! The central canvas region of the application.
//!
//! The canvas is responsible for :
//! - converting user input into camera mutations (pan, zoom),
//! - converting user input into scene mutations (drag-to-draw shapes),
//! - painting an adaptive grid background,
//! - painting every element in the scene,
//! - painting the in-progress shape preview when a tool is active.
//!
//! Submodules :
//! - [`panel`] — the entry point composed into the central `egui` panel.
//! - [`input`] — pure functions translating `egui` input into ops.
//! - [`grid`] — adaptive grid renderer.
//! - [`draw`] — renders scene elements + in-progress shape preview.

mod draw;
mod grid;
mod input;
mod panel;

pub use panel::show;
