//! The `Scene` is the in-memory representation of every element in a project.
//!
//! In v002 this is a simple `HashMap`. In later milestones it will be
//! backed by a CRDT document and complemented by a spatial index.

use std::collections::HashMap;

use bse_types::ElementId;

use crate::element::Element;

/// A collection of elements forming the visible content of a project.
#[derive(Clone, Debug, Default)]
pub struct Scene {
    elements: HashMap<ElementId, Element>,
}

impl Scene {
    /// Empty scene.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of elements in the scene.
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// `true` if there are no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Insert an element (replaces any existing element with the same id).
    pub fn insert(&mut self, element: Element) {
        self.elements.insert(element.id, element);
    }

    /// Remove an element by id, returning it if present.
    pub fn remove(&mut self, id: ElementId) -> Option<Element> {
        self.elements.remove(&id)
    }

    /// Borrow an element by id.
    #[must_use]
    pub fn get(&self, id: ElementId) -> Option<&Element> {
        self.elements.get(&id)
    }

    /// Mutably borrow an element by id.
    pub fn get_mut(&mut self, id: ElementId) -> Option<&mut Element> {
        self.elements.get_mut(&id)
    }

    /// Iterate over all elements (order is unspecified).
    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.elements.values()
    }

    /// Iterate over all elements sorted by Z order then by id.
    ///
    /// This is the order in which they should be rendered (back to front).
    #[must_use]
    pub fn iter_z_sorted(&self) -> Vec<&Element> {
        let mut all: Vec<&Element> = self.elements.values().collect();
        all.sort_by(|a, b| a.z.cmp(&b.z).then(a.id.as_uuid().cmp(&b.id.as_uuid())));
        all
    }
}

#[cfg(test)]
mod tests {
    use bse_types::{PeerId, Transform};

    use super::*;
    use crate::element::ElementKind;
    use crate::style::Style;

    fn elem(z: i32) -> Element {
        Element {
            id: ElementId::new(),
            kind: ElementKind::Rectangle {
                width: 10.0,
                height: 10.0,
            },
            style: Style::shape(),
            transform: Transform::IDENTITY,
            z,
            created_by: PeerId::new(),
            created_at: 0,
        }
    }

    #[test]
    fn insert_and_remove() {
        let mut scene = Scene::new();
        let e = elem(0);
        let id = e.id;
        scene.insert(e);
        assert_eq!(scene.len(), 1);
        scene.remove(id);
        assert!(scene.is_empty());
    }

    #[test]
    fn z_sorted_orders_by_z_then_id() {
        let mut scene = Scene::new();
        let a = elem(5);
        let b = elem(1);
        let c = elem(3);
        scene.insert(a);
        scene.insert(b);
        scene.insert(c);
        let sorted = scene.iter_z_sorted();
        assert_eq!(
            sorted.iter().map(|e| e.z).collect::<Vec<_>>(),
            vec![1, 3, 5]
        );
    }
}
