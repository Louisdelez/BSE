//! Public [`Quadtree`] facade.

use bse_types::Rect;

use crate::node::Node;

/// A spatial index over values of type `V`.
///
/// `V` is typically an [`bse_types::ElementId`] but the quadtree is
/// generic, so it can be reused for any `Copy + PartialEq` key.
///
/// See the [crate-level documentation][crate] for the full design.
#[derive(Debug, Clone)]
pub struct Quadtree<V: Copy + PartialEq> {
    /// Total bounds of the indexed space.
    bounds: Rect,
    /// Subdivide a leaf when it grows past this size.
    max_items_per_leaf: usize,
    /// Never subdivide deeper than this.
    max_depth: u32,
    /// Total number of `(value, bbox)` pairs currently stored.
    len: usize,
    /// Root node — initially an empty leaf.
    root: Node<V>,
}

impl<V: Copy + PartialEq> Quadtree<V> {
    /// Build a new quadtree spanning `bounds`.
    ///
    /// `max_items_per_leaf` (suggested default `16`) and `max_depth`
    /// (suggested default `8`) tune the tradeoff between memory and
    /// query speed :
    ///
    /// - Smaller `max_items_per_leaf` → more nodes, faster queries on
    ///   dense scenes, more memory.
    /// - Larger `max_depth` → finer spatial discrimination but deeper
    ///   recursion ; rarely useful past `10` for typical canvas sizes.
    ///
    /// `max_items_per_leaf` is internally clamped to at least `1` to
    /// avoid pathological infinite-split scenarios.
    #[must_use]
    pub fn new(bounds: Rect, max_items_per_leaf: usize, max_depth: u32) -> Self {
        Self {
            bounds,
            max_items_per_leaf: max_items_per_leaf.max(1),
            max_depth,
            len: 0,
            root: Node::new_leaf(),
        }
    }

    /// Total bounds of the index.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Insert a value associated with a bounding rectangle.
    ///
    /// If `bbox` falls outside the tree's bounds, the value is still
    /// stored (it ends up in the root straddlers / leaf), but query
    /// performance for it will degrade as everything that wants to find
    /// it must visit the root.
    pub fn insert(&mut self, value: V, bbox: Rect) {
        self.root.insert(
            value,
            bbox,
            self.bounds,
            0,
            self.max_items_per_leaf,
            self.max_depth,
        );
        self.len += 1;
    }

    /// Remove a value previously inserted with [`Self::insert`].
    /// Returns `true` if the value was present.
    ///
    /// The caller must provide the **same** `bbox` the value was
    /// inserted with — the quadtree does not remember insertion
    /// metadata beyond what is necessary for the spatial structure, so
    /// this is the cheapest way to locate the item.
    pub fn remove(&mut self, value: V, bbox: Rect) -> bool {
        let removed = self.root.remove(value, bbox, self.bounds);
        if removed {
            self.len -= 1;
        }
        removed
    }

    /// Return every value whose bbox intersects `region`.
    ///
    /// Values appear at most once in the returned vector — duplication
    /// across straddled leaves is impossible by construction.
    #[must_use]
    pub fn query(&self, region: Rect) -> Vec<V> {
        let mut out = Vec::new();
        self.root.query(region, self.bounds, &mut out);
        out
    }

    /// Total number of distinct values stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// `true` if no values are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Drop all values, keeping the bounds and tuning constants.
    pub fn clear(&mut self) {
        self.root = Node::new_leaf();
        self.len = 0;
    }
}
