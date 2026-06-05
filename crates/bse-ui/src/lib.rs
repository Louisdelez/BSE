//! Reusable UI components for the BSE desktop app.
//!
//! This crate is intentionally tiny in v002. As of v003 it will host
//! the egui widgets composing the BSE chrome : toolbar, properties
//! panel, presence list, modals.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod info;

pub use info::AppInfo;
