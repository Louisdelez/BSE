//! CRDT layer for BSE.
//!
//! This crate is the **abstraction boundary** between BSE and the
//! underlying CRDT library (yrs in v1, possibly Loro later). All higher
//! layers only depend on the [`CrdtBackend`] trait defined here, so
//! switching libraries is a single-point change.
//!
//! In v002 the crate exposes only the trait and a placeholder
//! `InMemoryBackend`. The real `yrs` integration arrives in v009
//! (see `docs/11-ROADMAP-EXECUTION/01-mvp.md`).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod backend;
mod error;
mod in_memory;

pub use backend::CrdtBackend;
pub use error::CrdtError;
pub use in_memory::InMemoryBackend;
