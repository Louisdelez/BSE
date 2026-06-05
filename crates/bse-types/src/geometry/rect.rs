//! Axis-aligned rectangle.

use serde::{Deserialize, Serialize};

use super::Vec2;

/// Axis-aligned rectangle defined by its `min` (top-left) and `max` (bottom-right)
/// corners in `y-down` space.
///
/// **Invariant** : `min.x <= max.x` and `min.y <= max.y`.
/// Use [`Rect::normalize`] to enforce it if you construct from untrusted points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// Top-left corner.
    pub min: Vec2,
    /// Bottom-right corner.
    pub max: Vec2,
}

impl Rect {
    /// An empty rectangle at the origin.
    pub const EMPTY: Self = Self {
        min: Vec2::ZERO,
        max: Vec2::ZERO,
    };

    /// Construct from `min` and `max` corners.
    ///
    /// Caller is responsible for the `min <= max` invariant.
    /// Use [`Self::from_two_points`] if the order is unknown.
    #[must_use]
    pub const fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Construct from any two points, normalizing so that
    /// `min` is the top-left and `max` the bottom-right.
    #[must_use]
    pub fn from_two_points(a: Vec2, b: Vec2) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }

    /// Construct from center and size.
    #[must_use]
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size / 2.0;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    /// Construct from top-left corner and size.
    #[must_use]
    pub fn from_origin_size(origin: Vec2, size: Vec2) -> Self {
        Self {
            min: origin,
            max: origin + size,
        }
    }

    /// Width.
    #[must_use]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height.
    #[must_use]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// `(width, height)` as a `Vec2`.
    #[must_use]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Geometric center.
    #[must_use]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    /// Area (`width * height`).
    #[must_use]
    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    /// Return a normalized copy with `min <= max`.
    #[must_use]
    pub fn normalize(&self) -> Self {
        Self::from_two_points(self.min, self.max)
    }

    /// Return `true` if `point` lies inside (inclusive on both edges).
    #[must_use]
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Return `true` if `other` is fully contained in `self`.
    #[must_use]
    pub fn contains_rect(&self, other: &Self) -> bool {
        self.contains(other.min) && self.contains(other.max)
    }

    /// Return `true` if `self` and `other` overlap (touching counts).
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Smallest rectangle containing both `self` and `other`.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Inflate by `margin` on every side. Negative shrinks.
    #[must_use]
    pub fn expand(&self, margin: f32) -> Self {
        let m = Vec2::splat(margin);
        Self {
            min: self.min - m,
            max: self.max + m,
        }
    }

    /// Inflate independently on `x` and `y`. Negative shrinks.
    #[must_use]
    pub fn expand_xy(&self, margin: Vec2) -> Self {
        Self {
            min: self.min - margin,
            max: self.max + margin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x1: f32, y1: f32, x2: f32, y2: f32) -> Rect {
        Rect::from_min_max(Vec2::new(x1, y1), Vec2::new(x2, y2))
    }

    #[test]
    fn size_and_center() {
        let rect = r(0.0, 0.0, 10.0, 6.0);
        assert_eq!(rect.size(), Vec2::new(10.0, 6.0));
        assert_eq!(rect.center(), Vec2::new(5.0, 3.0));
    }

    #[test]
    fn from_two_points_normalizes() {
        let rect = Rect::from_two_points(Vec2::new(5.0, 10.0), Vec2::new(0.0, 0.0));
        assert_eq!(rect.min, Vec2::ZERO);
        assert_eq!(rect.max, Vec2::new(5.0, 10.0));
    }

    #[test]
    fn contains_point() {
        let rect = r(0.0, 0.0, 10.0, 10.0);
        assert!(rect.contains(Vec2::new(5.0, 5.0)));
        assert!(rect.contains(Vec2::new(0.0, 0.0)));
        assert!(rect.contains(Vec2::new(10.0, 10.0)));
        assert!(!rect.contains(Vec2::new(-1.0, 0.0)));
        assert!(!rect.contains(Vec2::new(0.0, 11.0)));
    }

    #[test]
    fn intersects_cases() {
        let a = r(0.0, 0.0, 10.0, 10.0);
        assert!(a.intersects(&r(5.0, 5.0, 15.0, 15.0)));
        assert!(a.intersects(&r(10.0, 10.0, 20.0, 20.0))); // touching
        assert!(!a.intersects(&r(11.0, 11.0, 20.0, 20.0)));
    }

    #[test]
    fn union_covers_both() {
        let a = r(0.0, 0.0, 5.0, 5.0);
        let b = r(3.0, 3.0, 10.0, 10.0);
        let u = a.union(&b);
        assert_eq!(u, r(0.0, 0.0, 10.0, 10.0));
    }

    #[test]
    fn expand_grows() {
        let rect = r(5.0, 5.0, 10.0, 10.0);
        let exp = rect.expand(1.0);
        assert_eq!(exp, r(4.0, 4.0, 11.0, 11.0));
    }
}
