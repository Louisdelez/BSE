//! CRDT layer for BSE.
//!
//! This crate is the **abstraction boundary** between BSE and the
//! underlying CRDT library (yrs in v1, possibly Loro later). All higher
//! layers only depend on the [`CrdtBackend`] trait defined here, so
//! switching libraries is a single-point change.
//!
//! Two implementations are shipped :
//!
//! - [`InMemoryBackend`] — placeholder kept for single-user testing.
//! - [`YrsBackend`] — real Y-CRDT backed implementation (v009+).
//!
//! See `docs/11-ROADMAP-EXECUTION/01-mvp.md` for the full roadmap.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod backend;
mod error;
mod in_memory;
mod yrs_backend;

pub use backend::CrdtBackend;
pub use error::CrdtError;
pub use in_memory::InMemoryBackend;
pub use yrs_backend::YrsBackend;
