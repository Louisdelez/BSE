//! `yrs` (Y-CRDT) backed implementation of [`CrdtBackend`].
//!
//! # Storage strategy
//!
//! Elements are stored in a single root [`yrs::MapRef`] named `"elements"`.
//! Each element is keyed by the stringified [`ElementId`] and the value is
//! the JSON serialization of the [`Element`] held in a [`yrs::Any::String`].
//!
//! ## Why a JSON blob ?
//!
//! Yrs Maps converge field-by-field, but only if each field is its own
//! Map entry (or a nested shared type). Storing the element as a single
//! opaque JSON string is a deliberate **v009 simplification** :
//!
//! - It is trivial to (de)serialize by reusing the existing `serde`
//!   derives on [`Element`].
//! - The CRDT layer still converges at the **element level** : two peers
//!   concurrently editing different elements always merge cleanly.
//! - Concurrent edits **to the same element** resolve via last-write-wins
//!   on the entry, which is the standard yrs behaviour for `MapRef`.
//!
//! ## Known limits
//!
//! - Concurrent edits to the **same element** lose the field-level merge :
//!   if Alice moves the rectangle and Bob recolors it at the same time,
//!   one of the two changes is dropped.
//! - JSON is verbose. Update payloads scale with the full element body
//!   rather than the changed bytes.
//!
//! Both are accepted for v009. See the TODOs below for follow-up work.
//!
//! TODO(v009.1) : per-field CRDT operations rather than JSON-blob LWW.
//! TODO(v010) : awareness layer (cursors, selections, peer presence).

use bse_model::Element;
use bse_types::ElementId;
use yrs::updates::decoder::Decode;
use yrs::{Any, Doc, Map, MapRef, Out, ReadTxn, StateVector, Transact, TransactionMut, Update};

use crate::backend::CrdtBackend;
use crate::error::CrdtError;

/// Name of the root [`MapRef`] holding all elements.
const ELEMENTS_ROOT: &str = "elements";

/// A [`CrdtBackend`] backed by a real Y-CRDT document.
///
/// See the module documentation for the storage strategy and its limits.
pub struct YrsBackend {
    doc: Doc,
    elements: MapRef,
}

impl YrsBackend {
    /// Create a fresh, empty backend with a randomly assigned `client_id`.
    #[must_use]
    pub fn new() -> Self {
        let doc = Doc::new();
        let elements = doc.get_or_insert_map(ELEMENTS_ROOT);
        Self { doc, elements }
    }

    /// Create a backend with a specific yrs `client_id`.
    ///
    /// Useful in tests where deterministic conflict resolution is desired.
    #[must_use]
    pub fn with_client_id(client_id: u64) -> Self {
        let doc = Doc::with_client_id(client_id);
        let elements = doc.get_or_insert_map(ELEMENTS_ROOT);
        Self { doc, elements }
    }

    /// Borrow the underlying [`yrs::Doc`] (read-only access).
    ///
    /// Escape hatch for advanced integration (sync protocol, observers).
    /// Most callers should go through the [`CrdtBackend`] trait.
    #[must_use]
    pub fn doc(&self) -> &Doc {
        &self.doc
    }

    /// Encode the element as a JSON string ready to be stored in yrs.
    fn encode_element(element: &Element) -> Result<String, CrdtError> {
        serde_json::to_string(element)
            .map_err(|e| CrdtError::Internal(format!("element serialization failed : {e}")))
    }

    /// Decode an [`Out`] value (expected to be `Any::String`) back into an [`Element`].
    fn decode_element(out: &Out) -> Option<Element> {
        let Out::Any(Any::String(json)) = out else {
            return None;
        };
        serde_json::from_str(json.as_ref()).ok()
    }

    /// Stringified storage key for an [`ElementId`].
    fn key_for(id: ElementId) -> String {
        id.as_uuid().to_string()
    }

    /// Apply a mutating closure inside a fresh `transact_mut` scope.
    fn with_txn_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&MapRef, &mut TransactionMut<'_>) -> R,
    {
        let mut txn = self.doc.transact_mut();
        f(&self.elements, &mut txn)
    }
}

impl Default for YrsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for YrsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YrsBackend")
            .field("client_id", &self.doc.client_id())
            .field("element_count", &self.element_count())
            .finish_non_exhaustive()
    }
}

impl CrdtBackend for YrsBackend {
    fn element_count(&self) -> usize {
        let txn = self.doc.transact();
        self.elements.len(&txn) as usize
    }

    fn upsert_element(&mut self, element: Element) -> Result<(), CrdtError> {
        let json = Self::encode_element(&element)?;
        let key = Self::key_for(element.id);
        self.with_txn_mut(|map, txn| {
            map.insert(txn, key, json);
        });
        Ok(())
    }

    fn remove_element(&mut self, id: ElementId) -> Result<(), CrdtError> {
        let key = Self::key_for(id);
        self.with_txn_mut(|map, txn| {
            map.remove(txn, &key);
        });
        Ok(())
    }

    fn get_element(&self, id: ElementId) -> Option<Element> {
        let txn = self.doc.transact();
        let key = Self::key_for(id);
        let out = self.elements.get(&txn, &key)?;
        Self::decode_element(&out)
    }

    fn iter_elements(&self) -> Vec<Element> {
        let txn = self.doc.transact();
        self.elements
            .iter(&txn)
            .filter_map(|(_, out)| Self::decode_element(&out))
            .collect()
    }

    fn encode_snapshot(&self) -> Result<Vec<u8>, CrdtError> {
        let txn = self.doc.transact();
        // Full state, encoded with v2 — small payloads benefit less from v2
        // than batch snapshots, but consistency with `apply_remote_update`
        // matters more than the few bytes saved.
        let bytes = txn.encode_state_as_update_v2(&StateVector::default());
        Ok(bytes)
    }

    fn apply_remote_update(&mut self, bytes: &[u8]) -> Result<(), CrdtError> {
        let update = Update::decode_v2(bytes).map_err(|_| CrdtError::MalformedUpdate)?;
        let mut txn = self.doc.transact_mut();
        txn.apply_update(update)
            .map_err(|e| CrdtError::Internal(format!("apply_update failed : {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bse_model::{ElementKind, Style};
    use bse_types::{PeerId, Transform};

    use super::*;

    fn make_rect(width: f32, height: f32) -> Element {
        Element {
            id: ElementId::new(),
            kind: ElementKind::Rectangle { width, height },
            style: Style::shape(),
            transform: Transform::IDENTITY,
            z: 0,
            created_by: PeerId::new(),
            created_at: 0,
        }
    }

    #[test]
    fn empty_backend_has_zero_elements() {
        let backend = YrsBackend::new();
        assert_eq!(backend.element_count(), 0);
    }

    #[test]
    fn upsert_then_get_roundtrip() {
        let mut backend = YrsBackend::new();
        let element = make_rect(10.0, 20.0);
        let id = element.id;
        backend.upsert_element(element.clone()).expect("upsert");

        assert_eq!(backend.element_count(), 1);
        let fetched = backend.get_element(id).expect("present");
        assert_eq!(fetched, element);
    }

    #[test]
    fn upsert_replaces_existing_element() {
        let mut backend = YrsBackend::new();
        let mut element = make_rect(10.0, 20.0);
        let id = element.id;
        backend.upsert_element(element.clone()).expect("first");

        element.z = 42;
        backend.upsert_element(element.clone()).expect("second");

        assert_eq!(backend.element_count(), 1);
        let fetched = backend.get_element(id).expect("present");
        assert_eq!(fetched.z, 42);
    }

    #[test]
    fn remove_decrements_count() {
        let mut backend = YrsBackend::new();
        let element = make_rect(5.0, 5.0);
        let id = element.id;
        backend.upsert_element(element).expect("upsert");
        assert_eq!(backend.element_count(), 1);

        backend.remove_element(id).expect("remove");
        assert_eq!(backend.element_count(), 0);
        assert!(backend.get_element(id).is_none());
    }

    #[test]
    fn remove_missing_is_noop() {
        let mut backend = YrsBackend::new();
        let id = ElementId::new();
        backend.remove_element(id).expect("remove missing is ok");
        assert_eq!(backend.element_count(), 0);
    }

    #[test]
    fn get_missing_returns_none() {
        let backend = YrsBackend::new();
        assert!(backend.get_element(ElementId::new()).is_none());
    }
}
