//! Property-based tests for [`Quadtree`].
//!
//! Two invariants checked here :
//!
//! 1. `query(unbounded)` returns exactly all inserted values.
//! 2. For arbitrary inserts and an arbitrary query rect `R`, the
//!    quadtree's `query(R)` matches the brute-force "iterate every
//!    inserted bbox, keep those that intersect `R`" answer.

use bse_spatial::Quadtree;
use bse_types::{Rect, Vec2};
use proptest::prelude::*;

/// World half-extent used for the proptests.
const WORLD: f32 = 1000.0;

/// Generator for an arbitrary `Rect` whose corners lie in `[-WORLD, WORLD]`.
fn any_rect() -> impl Strategy<Value = Rect> {
    (
        -WORLD..=WORLD,
        -WORLD..=WORLD,
        0.0_f32..=200.0,
        0.0_f32..=200.0,
    )
        .prop_map(|(x, y, w, h)| Rect::from_min_max(Vec2::new(x, y), Vec2::new(x + w, y + h)))
}

/// Generator for a `(value, bbox)` pair.
fn any_item() -> impl Strategy<Value = (u32, Rect)> {
    (0u32..1_000, any_rect())
}

/// Bounds of the tree used in all proptests.
fn tree_bounds() -> Rect {
    Rect::from_min_max(Vec2::new(-WORLD, -WORLD), Vec2::new(WORLD, WORLD))
}

/// A rectangle that, by construction, contains every bbox we ever
/// generate (including straddlers and exact-boundary boxes).
fn unbounded() -> Rect {
    let huge = WORLD * 4.0;
    Rect::from_min_max(Vec2::new(-huge, -huge), Vec2::new(huge, huge))
}

/// Brute-force reference implementation : intersection by linear scan.
fn brute_force(items: &[(u32, Rect)], region: Rect) -> Vec<u32> {
    let mut out: Vec<u32> = items
        .iter()
        .filter(|(_, b)| region.intersects(b))
        .map(|(v, _)| *v)
        .collect();
    out.sort_unstable();
    out
}

proptest! {
    /// Property 1 : `query(unbounded_rect)` returns exactly all the
    /// inserted values (with their multiplicities since duplicate keys
    /// are allowed).
    #[test]
    fn query_unbounded_returns_all_inserts(items in proptest::collection::vec(any_item(), 0..200)) {
        let mut qt: Quadtree<u32> = Quadtree::new(tree_bounds(), 8, 6);
        for (v, b) in &items {
            qt.insert(*v, *b);
        }
        prop_assert_eq!(qt.len(), items.len());

        let mut got = qt.query(unbounded());
        got.sort_unstable();

        let mut expected: Vec<u32> = items.iter().map(|(v, _)| *v).collect();
        expected.sort_unstable();

        prop_assert_eq!(got, expected);
    }

    /// Property 2 : for any inserts and any query rect, the quadtree
    /// returns the same set (as a sorted vector) as the brute-force
    /// filter.
    #[test]
    fn query_matches_brute_force(
        items in proptest::collection::vec(any_item(), 0..200),
        query in any_rect(),
    ) {
        // Force unique keys so the comparison sees the same multiset.
        // (Brute force keeps duplicates ; so does the quadtree.) We
        // de-duplicate inputs here to avoid spurious diffs on equal
        // (id, bbox) pairs inserted twice — `remove` would behave
        // ambiguously, but `query` should still match brute force
        // including duplicates. We keep duplicates and compare sorted.
        let mut qt: Quadtree<u32> = Quadtree::new(tree_bounds(), 8, 6);
        for (v, b) in &items {
            qt.insert(*v, *b);
        }

        let mut got = qt.query(query);
        got.sort_unstable();

        let expected = brute_force(&items, query);

        prop_assert_eq!(got, expected);
    }

    /// Bonus : remove followed by query.
    /// After removing a random subset of inserted items, the remaining
    /// query result matches the brute-force filter on the residual set.
    #[test]
    fn remove_keeps_invariant(
        items in proptest::collection::vec(any_item(), 1..100),
        query in any_rect(),
        seed in 0u64..1_000,
    ) {
        let mut qt: Quadtree<u32> = Quadtree::new(tree_bounds(), 8, 6);
        for (v, b) in &items {
            qt.insert(*v, *b);
        }

        // Remove every other item using `seed` as the parity offset.
        let mut remaining: Vec<(u32, Rect)> = Vec::with_capacity(items.len());
        for (i, (v, b)) in items.iter().enumerate() {
            if (i as u64 + seed).is_multiple_of(2) {
                let was = qt.remove(*v, *b);
                prop_assert!(was, "every (v, b) we inserted must be removable");
            } else {
                remaining.push((*v, *b));
            }
        }

        prop_assert_eq!(qt.len(), remaining.len());

        let mut got = qt.query(query);
        got.sort_unstable();

        let expected = brute_force(&remaining, query);
        prop_assert_eq!(got, expected);
    }
}
