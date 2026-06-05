//! The central canvas region of the application.
//!
//! The canvas is responsible for :
//! - converting user input into camera mutations (pan, zoom),
//! - converting user input into scene mutations (drag-to-draw, click-to-place),
//! - painting an adaptive grid background,
//! - painting every element in the scene (including raster images),
//! - painting the in-progress shape preview when a tool is active.

mod draw;
mod grid;
mod input;
mod panel;

pub use draw::commit_image;
pub use panel::show;
