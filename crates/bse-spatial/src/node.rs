//! Internal node representation of the quadtree.
//!
//! A [`Node`] is either a [`Node::Leaf`] holding `(value, bbox)` pairs,
//! or a [`Node::Branch`] holding four child nodes plus the items whose
//! bbox straddles two or more child quadrants (the "straddlers").
//!
//! All public surface of the crate lives in [`crate::tree`] ; this module
//! is `pub(crate)`.

use bse_types::{Rect, Vec2};

/// Index of one of the four child quadrants of a branch node.
///
/// Ordered `NW, NE, SW, SE` — matches the layout of the design doc.
/// `y` grows downward (BSE convention).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Quadrant {
    /// North-West : low `x`, low `y`.
    Nw = 0,
    /// North-East : high `x`, low `y`.
    Ne = 1,
    /// South-West : low `x`, high `y`.
    Sw = 2,
    /// South-East : high `x`, high `y`.
    Se = 3,
}

impl Quadrant {
    /// All four quadrants in `NW, NE, SW, SE` order.
    pub(crate) const ALL: [Self; 4] = [Self::Nw, Self::Ne, Self::Sw, Self::Se];

    /// Bounding rectangle of this quadrant inside `parent_bounds`.
    pub(crate) fn sub_bounds(self, parent_bounds: Rect) -> Rect {
        let center = parent_bounds.center();
        let min = parent_bounds.min;
        let max = parent_bounds.max;
        match self {
            Self::Nw => Rect::from_min_max(Vec2::new(min.x, min.y), Vec2::new(center.x, center.y)),
            Self::Ne => Rect::from_min_max(Vec2::new(center.x, min.y), Vec2::new(max.x, center.y)),
            Self::Sw => Rect::from_min_max(Vec2::new(min.x, center.y), Vec2::new(center.x, max.y)),
            Self::Se => Rect::from_min_max(Vec2::new(center.x, center.y), Vec2::new(max.x, max.y)),
        }
    }
}

/// A node of the quadtree.
pub(crate) enum Node<V: Copy + PartialEq> {
    /// Terminal node : holds the actual `(value, bbox)` pairs.
    Leaf(Vec<(V, Rect)>),
    /// Internal node with four sub-quadrants plus the items whose bbox
    /// straddles more than one child quadrant (kept here to avoid
    /// duplication).
    Branch {
        /// Four child nodes, indexed by [`Quadrant`].
        children: Box<[Node<V>; 4]>,
        /// Items that don't fit cleanly in a single child quadrant.
        straddlers: Vec<(V, Rect)>,
    },
}

impl<V: Copy + PartialEq> Clone for Node<V> {
    fn clone(&self) -> Self {
        match self {
            Self::Leaf(items) => Self::Leaf(items.clone()),
            Self::Branch {
                children,
                straddlers,
            } => Self::Branch {
                children: Box::new([
                    children[0].clone(),
                    children[1].clone(),
                    children[2].clone(),
                    children[3].clone(),
                ]),
                straddlers: straddlers.clone(),
            },
        }
    }
}

impl<V: Copy + PartialEq> std::fmt::Debug for Node<V>
where
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(items) => f.debug_tuple("Leaf").field(items).finish(),
            Self::Branch {
                children,
                straddlers,
            } => f
                .debug_struct("Branch")
                .field("children", children)
                .field("straddlers", straddlers)
                .finish(),
        }
    }
}

impl<V: Copy + PartialEq> Node<V> {
    /// Create a fresh empty leaf.
    pub(crate) fn new_leaf() -> Self {
        Self::Leaf(Vec::new())
    }

    /// Recursive insert. `bounds` is *this* node's bounding rectangle.
    pub(crate) fn insert(
        &mut self,
        value: V,
        bbox: Rect,
        bounds: Rect,
        depth: u32,
        max_items: usize,
        max_depth: u32,
    ) {
        match self {
            Self::Leaf(items) => {
                items.push((value, bbox));
                if items.len() > max_items && depth < max_depth {
                    self.split(bounds, depth, max_items, max_depth);
                }
            }
            Self::Branch {
                children,
                straddlers,
            } => {
                if let Some(q) = unique_quadrant(bbox, bounds) {
                    let child_bounds = q.sub_bounds(bounds);
                    children[q as usize].insert(
                        value,
                        bbox,
                        child_bounds,
                        depth + 1,
                        max_items,
                        max_depth,
                    );
                } else {
                    straddlers.push((value, bbox));
                }
            }
        }
    }

    /// Subdivide a leaf into a branch with four empty children, then
    /// re-insert the items that fit into a single child quadrant.
    /// Items straddling several quadrants become straddlers.
    fn split(&mut self, bounds: Rect, depth: u32, max_items: usize, max_depth: u32) {
        // Take ownership of the leaf items.
        let items = match std::mem::replace(self, Self::new_leaf()) {
            Self::Leaf(v) => v,
            Self::Branch { .. } => unreachable!("split called on a Branch"),
        };

        let mut branch = Self::Branch {
            children: Box::new([
                Self::new_leaf(),
                Self::new_leaf(),
                Self::new_leaf(),
                Self::new_leaf(),
            ]),
            straddlers: Vec::new(),
        };

        for (v, b) in items {
            branch.insert(v, b, bounds, depth, max_items, max_depth);
        }

        *self = branch;
    }

    /// Recursive remove. Returns `true` if a `(value, bbox)` matching
    /// the arguments was found and dropped.
    pub(crate) fn remove(&mut self, value: V, bbox: Rect, bounds: Rect) -> bool {
        match self {
            Self::Leaf(items) => remove_first(items, value, bbox),
            Self::Branch {
                children,
                straddlers,
            } => {
                if let Some(q) = unique_quadrant(bbox, bounds) {
                    let child_bounds = q.sub_bounds(bounds);
                    children[q as usize].remove(value, bbox, child_bounds)
                } else {
                    remove_first(straddlers, value, bbox)
                }
            }
        }
    }

    /// Recursive query : push every value whose bbox intersects `region`
    /// into `out`. Because each `(value, bbox)` lives in exactly one
    /// node, no de-dup is required.
    pub(crate) fn query(&self, region: Rect, bounds: Rect, out: &mut Vec<V>) {
        // Skip the whole subtree if the region cannot touch our bounds.
        if !region.intersects(&bounds) {
            return;
        }
        match self {
            Self::Leaf(items) => {
                for (v, b) in items {
                    if region.intersects(b) {
                        out.push(*v);
                    }
                }
            }
            Self::Branch {
                children,
                straddlers,
            } => {
                for (v, b) in straddlers {
                    if region.intersects(b) {
                        out.push(*v);
                    }
                }
                for q in Quadrant::ALL {
                    let child_bounds = q.sub_bounds(bounds);
                    children[q as usize].query(region, child_bounds, out);
                }
            }
        }
    }
}

/// Remove the first `(v, b)` such that `v == value && b == bbox`.
/// Returns `true` if a match was found.
fn remove_first<V: Copy + PartialEq>(items: &mut Vec<(V, Rect)>, value: V, bbox: Rect) -> bool {
    if let Some(pos) = items.iter().position(|(v, b)| *v == value && *b == bbox) {
        items.swap_remove(pos);
        true
    } else {
        false
    }
}

/// If `bbox` is fully contained in exactly one of the four child
/// quadrants of `bounds`, return that quadrant. Otherwise return `None`
/// (it straddles two or more).
fn unique_quadrant(bbox: Rect, bounds: Rect) -> Option<Quadrant> {
    let cx = (bounds.min.x + bounds.max.x) * 0.5;
    let cy = (bounds.min.y + bounds.max.y) * 0.5;
    let in_west = bbox.max.x <= cx;
    let in_east = bbox.min.x >= cx;
    let in_north = bbox.max.y <= cy;
    let in_south = bbox.min.y >= cy;

    match (in_west, in_east, in_north, in_south) {
        (true, _, true, _) => Some(Quadrant::Nw),
        (_, true, true, _) => Some(Quadrant::Ne),
        (true, _, _, true) => Some(Quadrant::Sw),
        (_, true, _, true) => Some(Quadrant::Se),
        _ => None,
    }
}
