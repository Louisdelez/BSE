//! Error type for CRDT operations.

use thiserror::Error;

/// An error returned by a [`crate::CrdtBackend`] operation.
#[derive(Debug, Error)]
pub enum CrdtError {
    /// The incoming update could not be decoded.
    #[error("malformed CRDT update")]
    MalformedUpdate,

    /// Applying the update failed because of an internal invariant violation.
    #[error("CRDT internal error : {0}")]
    Internal(String),

    /// Feature not yet implemented at this milestone.
    #[error("not yet implemented : {0}")]
    NotImplemented(&'static str),
}
