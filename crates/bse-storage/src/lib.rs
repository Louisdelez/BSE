//! Local persistence for BSE.
//!
//! Stores project snapshots in `SQLite` and asset binaries on the filesystem.
//! The real implementation lands in v011 (post-MVP demo) ; v002 only
//! defines the trait.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use thiserror::Error;

/// Errors produced by storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    /// The underlying I/O operation failed.
    #[error("I/O error : {0}")]
    Io(String),

    /// Feature not yet implemented at this milestone.
    #[error("not yet implemented : {0}")]
    NotImplemented(&'static str),
}

/// Local storage trait.
///
/// Implementations persist project snapshots and serve them back on
/// app restart. The signature deliberately keeps the API tiny ;
/// fancier features (asset cache, WAL replay) will be added as
/// separate traits in later milestones.
pub trait LocalStorage: Send + Sync {
    /// Persist a project snapshot under a string key (typically the
    /// project id as a UUID string).
    fn save_snapshot(&mut self, key: &str, bytes: &[u8]) -> Result<(), StorageError>;

    /// Load a previously-saved snapshot, returning `None` if missing.
    fn load_snapshot(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
}
