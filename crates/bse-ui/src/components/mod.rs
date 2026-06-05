//! Reusable widgets built on top of egui primitives.
//!
//! Every widget here pulls its colors, fonts and motion tokens from
//! [`crate::theme`] — feature code stays declarative.

mod avatar;
mod card;
mod modal;
mod pill_button;
mod status_pill;

pub use avatar::{avatar, avatar_stack};
pub use card::{Card, CardVariant, card_button};
pub use modal::{Modal, ModalResponse, show_modal};
pub use pill_button::{PillButton, PillVariant};
pub use status_pill::{PillTone, StatusPill};
