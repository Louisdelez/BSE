//! 2D vector.
//!
//! Thin wrapper over [`glam::Vec2`] adding `serde` and BSE conventions.
//! All BSE coordinates are `y-down` (origin top-left, like screen space).

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// 2D vector in `f32`, with `y` axis pointing down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct Vec2 {
    /// X component (rightward).
    pub x: f32,
    /// Y component (downward).
    pub y: f32,
}

impl Vec2 {
    /// `(0, 0)`.
    pub const ZERO: Self = Self::new(0.0, 0.0);
    /// `(1, 1)`.
    pub const ONE: Self = Self::new(1.0, 1.0);
    /// `(1, 0)`.
    pub const X: Self = Self::new(1.0, 0.0);
    /// `(0, 1)`.
    pub const Y: Self = Self::new(0.0, 1.0);

    /// Construct from components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Construct with both components equal to `v`.
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    /// Euclidean length.
    #[must_use]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Euclidean length squared (cheaper than `length`).
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Distance to another point.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    /// Dot product.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Component-wise minimum.
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    /// Component-wise maximum.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    /// Linear interpolation.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

// ─── Conversions ──────────────────────────────────────────────────────────────

impl From<glam::Vec2> for Vec2 {
    fn from(v: glam::Vec2) -> Self {
        Self::new(v.x, v.y)
    }
}

impl From<Vec2> for glam::Vec2 {
    fn from(v: Vec2) -> Self {
        Self::new(v.x, v.y)
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

// ─── Operators ────────────────────────────────────────────────────────────────

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a + b, Vec2::new(4.0, 6.0));
        assert_eq!(b - a, Vec2::new(2.0, 2.0));
        assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
    }

    #[test]
    fn length_and_distance() {
        assert!((Vec2::new(3.0, 4.0).length() - 5.0).abs() < 1e-5);
        assert!((Vec2::ZERO.distance(Vec2::new(0.0, 1.0)) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn lerp_midpoint() {
        let m = Vec2::ZERO.lerp(Vec2::new(2.0, 4.0), 0.5);
        assert_eq!(m, Vec2::new(1.0, 2.0));
    }
}
