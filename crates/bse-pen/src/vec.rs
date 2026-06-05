//! Internal 2D vector helpers used by the stroke algorithm.
//!
//! All helpers operate on [`bse_types::Vec2`] and are direct ports of
//! the small `vec.ts` module from perfect-freehand. They are kept
//! `pub(crate)` because [`bse_types::Vec2`] already exposes the parts
//! needed by external callers (`+`, `-`, `*`, `lerp`, `length`, ...).

use bse_types::Vec2;

/// Tolerance used to compare points for equality. Matches the
/// `0.0001` constant used internally by perfect-freehand.
pub(crate) const EQUAL_EPSILON: f32 = 1.0e-4;

/// `PI` with a tiny offset, mirroring perfect-freehand's `FIXED_PI`.
///
/// Some renderers produce gaps when rotating by exactly `PI`; the
/// offset hides that artifact at no visible cost.
pub(crate) const FIXED_PI: f32 = core::f32::consts::PI + 0.000_1;

/// Dot product of two vectors.
#[inline]
#[must_use]
pub(crate) fn dpr(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

/// Perpendicular (right-handed in y-down): `[y, -x]`.
#[inline]
#[must_use]
pub(crate) fn per(a: Vec2) -> Vec2 {
    Vec2::new(a.y, -a.x)
}

/// Negation of a vector.
#[inline]
#[must_use]
pub(crate) fn neg(a: Vec2) -> Vec2 {
    Vec2::new(-a.x, -a.y)
}

/// Unit vector. Returns the zero vector if `a` has zero length.
#[inline]
#[must_use]
pub(crate) fn uni(a: Vec2) -> Vec2 {
    let len = a.length();
    if len > 0.0 { a / len } else { Vec2::ZERO }
}

/// Project `a` along direction `b` by scalar distance `c`.
#[inline]
#[must_use]
pub(crate) fn prj(a: Vec2, b: Vec2, c: f32) -> Vec2 {
    a + b * c
}

/// Squared distance (cheaper than `distance` when only comparing).
#[inline]
#[must_use]
pub(crate) fn dist2(a: Vec2, b: Vec2) -> f32 {
    let d = a - b;
    d.x * d.x + d.y * d.y
}

/// Rotate `a` around center `c` by `r` radians.
#[inline]
#[must_use]
pub(crate) fn rot_around(a: Vec2, c: Vec2, r: f32) -> Vec2 {
    let (sin, cos) = r.sin_cos();
    let px = a.x - c.x;
    let py = a.y - c.y;
    Vec2::new(px * cos - py * sin + c.x, px * sin + py * cos + c.y)
}

/// Component-wise equality within [`EQUAL_EPSILON`].
#[inline]
#[must_use]
pub(crate) fn approx_eq(a: Vec2, b: Vec2) -> bool {
    (a.x - b.x).abs() < EQUAL_EPSILON && (a.y - b.y).abs() < EQUAL_EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perpendicular_is_orthogonal() {
        let v = Vec2::new(3.0, 4.0);
        assert!(dpr(v, per(v)).abs() < 1e-5);
    }

    #[test]
    fn uni_of_axis() {
        let u = uni(Vec2::new(5.0, 0.0));
        assert!((u.x - 1.0).abs() < 1e-6);
        assert!(u.y.abs() < 1e-6);
    }

    #[test]
    fn uni_of_zero_is_zero() {
        assert_eq!(uni(Vec2::ZERO), Vec2::ZERO);
    }

    #[test]
    fn rot_around_quarter_turn() {
        // Rotate (1, 0) by +90° around the origin.
        // y-down convention: clockwise on screen.
        let r = rot_around(
            Vec2::new(1.0, 0.0),
            Vec2::ZERO,
            core::f32::consts::FRAC_PI_2,
        );
        assert!(r.x.abs() < 1e-5);
        assert!((r.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn prj_along_unit() {
        let r = prj(Vec2::ZERO, Vec2::new(1.0, 0.0), 3.0);
        assert_eq!(r, Vec2::new(3.0, 0.0));
    }
}
