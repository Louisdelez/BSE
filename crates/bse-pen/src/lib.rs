//! Pressure-sensitive freehand stroke algorithm.
//!
//! Rust port of [perfect-freehand](https://github.com/steveruizok/perfect-freehand)
//! by Steve Ruiz, adapted to BSE's coordinate system and types.
//!
//! Given a sequence of `(x, y, pressure)` input points, this crate
//! produces a closed outline polygon (a `Vec<bse_types::Vec2>`) suitable
//! for filling with a tessellator or for SVG export. The polygon is
//! closed implicitly: the last vertex connects back to the first.
//!
//! # Example
//!
//! ```
//! use bse_pen::{InputPoint, StrokeOptions, get_stroke};
//!
//! let points = vec![
//!     InputPoint::new(0.0, 0.0, 0.5),
//!     InputPoint::new(10.0, 5.0, 0.6),
//!     InputPoint::new(20.0, 0.0, 0.7),
//! ];
//! let outline = get_stroke(&points, &StrokeOptions::default());
//! assert!(!outline.is_empty());
//! ```
//!
//! # Coordinate convention
//!
//! Like the original, BSE uses a **y-down** coordinate system where
//! the origin is at the top-left. All angles in this crate are
//! consistent with that convention.
//!
//! # Algorithmic shortcuts vs. the TypeScript original
//!
//! The v006 port intentionally drops a few rarely-used knobs from the
//! original to keep the API flat and `f32`-typed. They are tagged
//! `TODO(v006.1)` in the code:
//!
//! - **Custom easing functions.** The pressure body curve and the
//!   start/end taper easings are hard-coded to the TypeScript
//!   defaults (`t`, `t * (2 - t)`, `(t - 1)^3 + 1`).
//! - **Per-end cap toggles.** Tapered ends do not draw a cap; flat
//!   caps are not exposed.
//! - **`last: true` completion flag.** Strokes are always treated as
//!   in-progress: the final input sample is used as-is.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod options;
mod outline;
mod stroke_points;
mod vec;

pub use options::{InputPoint, StrokeOptions};

use bse_types::Vec2;

/// Compute the outline polygon for the given input points.
///
/// Returns a closed polygon as `Vec<bse_types::Vec2>` where the last
/// vertex implicitly connects back to the first. An empty input or
/// `options.size <= 0.0` yields an empty `Vec`.
#[must_use]
pub fn get_stroke(points: &[InputPoint], options: &StrokeOptions) -> Vec<Vec2> {
    let stroke_points = stroke_points::get_stroke_points(points, options);
    outline::get_stroke_outline_points(&stroke_points, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> StrokeOptions {
        StrokeOptions::default()
    }

    #[test]
    fn empty_input_returns_empty_outline() {
        let out = get_stroke(&[], &options());
        assert!(out.is_empty(), "empty input should produce no outline");
    }

    #[test]
    fn single_point_returns_a_dot() {
        let pts = [InputPoint::new(5.0, 7.0, 0.5)];
        let out = get_stroke(&pts, &options());
        assert!(!out.is_empty(), "a single point should render as a dot");

        // All dot vertices should lie within `size` units of the input,
        // since the radius is at most `size / 2` (no thinning push).
        let max_dist: f32 = out
            .iter()
            .map(|p| p.distance(Vec2::new(5.0, 7.0)))
            .fold(0.0_f32, f32::max);
        assert!(
            max_dist <= options().size,
            "dot wandered too far: {max_dist}"
        );
    }

    #[test]
    fn two_point_straight_line_has_left_and_right_sides() {
        let pts = [
            InputPoint::new(0.0, 0.0, 0.5),
            InputPoint::new(100.0, 0.0, 0.5),
        ];
        let out = get_stroke(&pts, &options());
        assert!(
            out.len() >= 4,
            "straight line outline is too small: {}",
            out.len()
        );

        // Outline should span both sides of the y axis.
        let min_y = out.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = out.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        assert!(min_y < 0.0, "outline should reach above the centerline");
        assert!(max_y > 0.0, "outline should reach below the centerline");
    }

    #[test]
    fn curved_stroke_produces_non_degenerate_polygon() {
        // A gentle S-curve so we get genuinely curved geometry.
        let pts: Vec<InputPoint> = (0..20_u32)
            .map(|i| {
                let t = f32::from(u16::try_from(i).unwrap_or(0)) / 19.0;
                InputPoint::new(t * 100.0, (t * std::f32::consts::TAU).sin() * 20.0, 0.5)
            })
            .collect();
        let out = get_stroke(&pts, &options());
        assert!(
            out.len() >= 10,
            "curved stroke should produce a richer polygon, got {}",
            out.len()
        );

        // Verify the polygon has non-zero area, ie. it is not a line.
        let area = polygon_area(&out);
        assert!(area > 1.0, "outline area too small: {area}");
    }

    #[test]
    fn negative_or_zero_size_produces_empty_output() {
        let pts = [
            InputPoint::new(0.0, 0.0, 0.5),
            InputPoint::new(10.0, 0.0, 0.5),
        ];
        let mut opts = options();
        opts.size = 0.0;
        assert!(get_stroke(&pts, &opts).is_empty());
        opts.size = -1.0;
        assert!(get_stroke(&pts, &opts).is_empty());
    }

    #[test]
    fn outline_is_finite() {
        let pts = [
            InputPoint::new(0.0, 0.0, 0.5),
            InputPoint::new(50.0, 50.0, 0.7),
            InputPoint::new(100.0, 0.0, 0.5),
        ];
        let out = get_stroke(&pts, &options());
        assert!(
            out.iter().all(|p| p.x.is_finite() && p.y.is_finite()),
            "outline should contain only finite coordinates"
        );
    }

    /// Shoelace formula. Used only by tests to assert the outline has
    /// non-zero area.
    fn polygon_area(poly: &[Vec2]) -> f32 {
        if poly.len() < 3 {
            return 0.0;
        }
        let mut sum = 0.0_f32;
        for i in 0..poly.len() {
            let a = poly[i];
            let b = poly[(i + 1) % poly.len()];
            sum += a.x * b.y - b.x * a.y;
        }
        (sum * 0.5).abs()
    }
}
