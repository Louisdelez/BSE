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

    /// Encode the full state for sharing with a new peer.
    fn encode_snapshot(&self) -> Result<Vec<u8>, CrdtError>;

    /// Apply a binary update produced by another peer.
    fn apply_remote_update(&mut self, bytes: &[u8]) -> Result<(), CrdtError>;
}
