//! Canvas state and input handling.
//!
//! The canvas owns the [`Camera`](bse_model::Camera), the active tool,
//! and the current selection. It does **not** own the scene (that lives
//! in the CRDT layer) ; it dispatches events that mutate the scene.
//!
//! v002 ships only the [`CanvasState`] struct with camera handling.
//! Tools and selection arrive in v004 and v005.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod state;
mod tool;

pub use state::{CanvasState, ToolState};
pub use tool::ToolKind;
