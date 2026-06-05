//! Placeholder in-memory backend (no actual CRDT semantics).
//!
//! This is good enough for single-user testing in v002–v008. It does
//! **not** support concurrent edits and **panics** on snapshot/update
//! encoding. The real CRDT backend (`yrs`) replaces this in v009.

use bse_model::{Element, Scene};
use bse_types::ElementId;

use crate::backend::CrdtBackend;
use crate::error::CrdtError;

/// A minimal in-memory store of elements, used until yrs is integrated.
#[derive(Debug, Default)]
pub struct InMemoryBackend {
    scene: Scene,
}

impl InMemoryBackend {
    /// Empty backend.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Read-only access to the underlying [`Scene`].
    #[must_use]
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
}

impl CrdtBackend for InMemoryBackend {
    fn element_count(&self) -> usize {
        self.scene.len()
    }

    fn upsert_element(&mut self, element: Element) -> Result<(), CrdtError> {
        self.scene.insert(element);
        Ok(())
    }

    fn remove_element(&mut self, id: ElementId) -> Result<(), CrdtError> {
        self.scene.remove(id);
        Ok(())
    }

    fn get_element(&self, id: ElementId) -> Option<Element> {
        self.scene.get(id).cloned()
    }

    fn iter_elements(&self) -> Vec<Element> {
        self.scene.iter().cloned().collect()
    }

    fn encode_snapshot(&self) -> Result<Vec<u8>, CrdtError> {
        Err(CrdtError::NotImplemented(
            "snapshot encoding lands with yrs in v009",
        ))
    }

    fn apply_remote_update(&mut self, _bytes: &[u8]) -> Result<(), CrdtError> {
        Err(CrdtError::NotImplemented(
            "remote updates land with yrs in v009",
        ))
    }
}
