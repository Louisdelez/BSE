//! HTTP request handlers.
//!
//! Each route gets its own module; the [`crate::routes`] module wires
//! them onto the axum router.

pub mod health;
pub mod info;
