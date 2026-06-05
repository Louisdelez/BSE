//! Reusable widgets built on top of egui primitives.
//!
//! Every widget here pulls its colors, fonts and motion tokens from
//! [`crate::theme`] — feature code stays declarative.

mod card;
mod modal;
mod pill_button;

pub use card::{Card, CardVariant, card_button};
pub use modal::{Modal, ModalResponse, show_modal};
pub use pill_button::{PillButton, PillVariant};
