//! Unit-style integration tests for the public [`Quadtree`] API.
//!
//! Lives in `tests/` so that it does not inflate the per-file line
//! budget of the `src/` modules.

use bse_spatial::Quadtree;
use bse_types::{Rect, Vec2};

/// Convenience constructor.
fn r(x1: f32, y1: f32, x2: f32, y2: f32) -> Rect {
    Rect::from_min_max(Vec2::new(x1, y1), Vec2::new(x2, y2))
}

/// Standard tree bounds used in most tests.
fn world() -> Rect {
    r(-1000.0, -1000.0, 1000.0, 1000.0)
}

#[test]
fn empty_tree_query_returns_nothing() {
    let qt: Quadtree<u32> = Quadtree::new(world(), 16, 8);
    assert!(qt.is_empty());
    assert_eq!(qt.len(), 0);
    assert!(qt.query(world()).is_empty());
    assert!(qt.query(r(0.0, 0.0, 1.0, 1.0)).is_empty());
}

#[test]
fn single_insert_and_query() {
    let mut qt: Quadtree<u32> = Quadtree::new(world(), 16, 8);
    qt.insert(42, r(10.0, 10.0, 20.0, 20.0));

    assert_eq!(qt.len(), 1);
    assert!(!qt.is_empty());

    let hits = qt.query(r(0.0, 0.0, 100.0, 100.0));
    assert_eq!(hits, vec![42]);
}

#[test]
fn query_inside_outside_and_overlapping_a_leaf() {
    let mut qt: Quadtree<u32> = Quadtree::new(world(), 16, 8);

    // Three boxes, well separated, all in the NE quadrant of the world.
    qt.insert(1, r(100.0, 100.0, 200.0, 200.0));
    qt.insert(2, r(300.0, 300.0, 400.0, 400.0));
    qt.insert(3, r(500.0, 500.0, 600.0, 600.0));

    // Query fully inside the first bbox : only #1.
    let hits = qt.query(r(120.0, 120.0, 180.0, 180.0));
    assert_eq!(sorted(hits), vec![1]);

    // Query outside everything : empty.
    let hits = qt.query(r(-500.0, -500.0, -400.0, -400.0));
    assert!(hits.is_empty());

    // Query overlapping two : #1 and #2.
    let hits = qt.query(r(150.0, 150.0, 350.0, 350.0));
    assert_eq!(sorted(hits), vec![1, 2]);
}

#[test]
fn remove_existing_and_non_existing() {
    let mut qt: Quadtree<u32> = Quadtree::new(world(), 16, 8);
    let bbox = r(0.0, 0.0, 10.0, 10.0);
    qt.insert(7, bbox);

    assert_eq!(qt.len(), 1);

    // Wrong bbox → not removed.
    assert!(!qt.remove(7, r(0.0, 0.0, 5.0, 5.0)));
    assert_eq!(qt.len(), 1);

    // Wrong id → not removed.
    assert!(!qt.remove(8, bbox));
    assert_eq!(qt.len(), 1);

    // Correct → removed.
    assert!(qt.remove(7, bbox));
    assert_eq!(qt.len(), 0);
    assert!(qt.is_empty());

    // Removing again → false, len stays at 0.
    assert!(!qt.remove(7, bbox));
    assert_eq!(qt.len(), 0);
}

#[test]
fn subdivision_kicks_in_past_threshold() {
    // Tight thresholds so we can observe the split easily.
    let max_items = 4;
    let max_depth = 4;
    let mut qt: Quadtree<u32> = Quadtree::new(world(), max_items, max_depth);

    // Insert 20 well-distributed boxes that fall cleanly into the four
    // child quadrants. With max_items = 4 the root must split.
    let coords = [
        // NW : 5 items
        (-900.0, -900.0),
        (-800.0, -800.0),
        (-700.0, -700.0),
        (-600.0, -600.0),
        (-500.0, -500.0),
        // NE : 5 items
        (100.0, -900.0),
        (200.0, -800.0),
        (300.0, -700.0),
        (400.0, -600.0),
        (500.0, -500.0),
        // SW : 5 items
        (-900.0, 100.0),
        (-800.0, 200.0),
        (-700.0, 300.0),
        (-600.0, 400.0),
        (-500.0, 500.0),
        // SE : 5 items
        (100.0, 100.0),
        (200.0, 200.0),
        (300.0, 300.0),
        (400.0, 400.0),
        (500.0, 500.0),
    ];
    for (i, (x, y)) in coords.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        qt.insert(i as u32, r(*x, *y, *x + 10.0, *y + 10.0));
    }

    assert_eq!(qt.len(), 20);

    // Sanity : every inserted value is found by a global query.
    let mut all = qt.query(world());
    all.sort_unstable();
    let expected: Vec<u32> = (0..20).collect();
    assert_eq!(all, expected);

    // Query that hits only the SE quadrant — expect items 15..20.
    let hits = qt.query(r(50.0, 50.0, 600.0, 600.0));
    assert_eq!(sorted(hits), vec![15, 16, 17, 18, 19]);
}

#[test]
fn clear_drops_everything_keeps_bounds() {
    let mut qt: Quadtree<u32> = Quadtree::new(world(), 4, 4);
    for i in 0..50u32 {
        #[allow(clippy::cast_precision_loss)]
        let x = i as f32;
        qt.insert(i, r(x, x, x + 1.0, x + 1.0));
    }
    assert_eq!(qt.len(), 50);
    qt.clear();
    assert_eq!(qt.len(), 0);
    assert!(qt.is_empty());
    assert!(qt.query(world()).is_empty());
    assert_eq!(qt.bounds(), world());

    // Still usable after clear.
    qt.insert(99, r(0.0, 0.0, 1.0, 1.0));
    assert_eq!(qt.query(world()), vec![99]);
}

#[test]
fn straddling_bbox_is_not_duplicated() {
    // Force subdivision and put a bbox straddling all four quadrants
    // (around the world origin). It must show up exactly once.
    let mut qt: Quadtree<u32> = Quadtree::new(world(), 4, 4);

    // 5 small boxes in NE → force split.
    for i in 0..5u32 {
        #[allow(clippy::cast_precision_loss)]
        let x = 100.0 + (i as f32) * 10.0;
        qt.insert(i, r(x, x, x + 5.0, x + 5.0));
    }

    // Straddles the world center.
    qt.insert(100, r(-50.0, -50.0, 50.0, 50.0));

    let hits = qt.query(world());
    let count = hits.iter().filter(|v| **v == 100).count();
    assert_eq!(count, 1, "straddler must not be duplicated");

    // And a tight query also returns it exactly once.
    let hits = qt.query(r(-1.0, -1.0, 1.0, 1.0));
    assert_eq!(hits.iter().filter(|v| **v == 100).count(), 1);
}

#[test]
fn touching_query_includes_boundary() {
    // Rect intersection in BSE is inclusive.
    let mut qt: Quadtree<u32> = Quadtree::new(world(), 16, 8);
    qt.insert(1, r(0.0, 0.0, 10.0, 10.0));

    // Query exactly touching corner.
    let hits = qt.query(r(10.0, 10.0, 20.0, 20.0));
    assert_eq!(hits, vec![1]);
}

fn sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    v.sort();
    v
}
