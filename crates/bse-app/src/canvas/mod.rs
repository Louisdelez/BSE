//! The central canvas region of the application.
//!
//! The canvas is responsible for :
//! - converting user input into camera mutations (pan, zoom),
//! - converting user input into document mutations (drag-to-draw,
//!   click-to-place, inline text edit),
//! - painting an adaptive grid background,
//! - painting every element in the document (including raster images),
//! - painting the in-progress shape preview when a tool is active,
//! - rendering the inline text-edit overlay when appropriate.

mod draw;
mod grid;
mod input;
mod panel;
mod text_edit;

pub use draw::commit_image;
pub use panel::show;
