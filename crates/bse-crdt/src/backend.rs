//! The [`CrdtBackend`] trait.

use bse_model::Element;
use bse_types::ElementId;

use crate::error::CrdtError;

/// Abstraction over a CRDT implementation.
///
/// A backend can be queried for the current state, mutated through
/// strongly-typed operations, or fed remote binary updates produced
/// by a peer.
pub trait CrdtBackend: Send + Sync + 'static {
    /// Number of elements currently in the document.
    fn element_count(&self) -> usize;

    /// Insert or replace an element.
    fn upsert_element(&mut self, element: Element) -> Result<(), CrdtError>;

    /// Remove an element by id. No-op if not present.
    fn remove_element(&mut self, id: ElementId) -> Result<(), CrdtError>;

    /// Borrow an element.
    fn get_element(&self, id: ElementId) -> Option<Element>;

    /// Snapshot every element currently in the document.
    ///
    /// Iteration order is unspecified ; callers that need z-order or
    /// time order sort the result themselves. This is the "read view"
    /// used by the renderer.
    fn iter_elements(&self) -> Vec<Element>;

    /// Encode the full state for sharing with a new peer.
    fn encode_snapshot(&self) -> Result<Vec<u8>, CrdtError>;

    /// Encode the document's current state vector. This is what a
    /// remote peer sends back so we can ship them an *incremental*
    /// update (see [`Self::encode_update_since`]) rather than the
    /// whole document.
    fn state_vector(&self) -> Result<Vec<u8>, CrdtError>;

    /// Encode the bytes needed to bring a peer with the given
    /// state vector up to date. The default implementation falls
    /// back to a full snapshot ; the Yrs backend overrides it with
    /// a real incremental update.
    ///
    /// Pass an empty slice to fall back to a full snapshot.
    fn encode_update_since(&self, remote_sv: &[u8]) -> Result<Vec<u8>, CrdtError> {
        let _ = remote_sv;
        self.encode_snapshot()
    }

    /// Apply a binary update produced by another peer.
    fn apply_remote_update(&mut self, bytes: &[u8]) -> Result<(), CrdtError>;
}
