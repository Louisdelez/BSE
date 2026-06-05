//! Spatial indexing primitives.
//!
//! BSE uses a [`Quadtree`] to answer "which elements are visible in this
//! viewport ?" in `O(log N + K)` time, where `K` is the number of
//! results. This is the difference between a smooth 60 FPS canvas
//! and a laggy 10 FPS one once a scene has more than a few hundred
//! elements.
//!
//! The data structure is **generic** over the value type `V`. It is
//! typically instantiated with [`bse_types::ElementId`], but works for
//! any `Copy + PartialEq` key.
//!
//! # Example
//!
//! ```
//! use bse_spatial::Quadtree;
//! use bse_types::{Rect, Vec2};
//!
//! let bounds = Rect::from_min_max(Vec2::new(-1000.0, -1000.0), Vec2::new(1000.0, 1000.0));
//! let mut qt: Quadtree<u32> = Quadtree::new(bounds, 16, 8);
//!
//! qt.insert(1, Rect::from_min_max(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)));
//! qt.insert(2, Rect::from_min_max(Vec2::new(100.0, 100.0), Vec2::new(120.0, 120.0)));
//!
//! let hits = qt.query(Rect::from_min_max(Vec2::new(-5.0, -5.0), Vec2::new(50.0, 50.0)));
//! assert_eq!(hits, vec![1]);
//! ```
//!
//! # Design notes
//!
//! - A bbox that straddles several child quadrants is **kept in the parent
//!   branch** rather than duplicated into multiple leaves. This makes
//!   `query` naturally de-duplicating (no value appears in more than
//!   one node) and removes the need for an external "value to leaf" map.
//! - `remove` does **not** rebalance the tree : empty branches stay
//!   subdivided until [`Quadtree::clear`] or a full rebuild. This keeps
//!   the code straightforward — rebalancing is an explicit caller
//!   decision (e.g. periodic rebuild).
//! - Subdivision happens when a leaf exceeds `max_items_per_leaf` **and**
//!   its depth is `< max_depth`.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod node;
mod tree;

pub use tree::Quadtree;
