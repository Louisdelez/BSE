//! 2D affine transformation (translation + rotation + scale).

use serde::{Deserialize, Serialize};

use super::Vec2;

/// 2D affine transform : translation, rotation (radians) and non-uniform scale.
///
/// Operations apply in the order `scale → rotate → translate`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Translation component.
    pub translation: Vec2,
    /// Rotation in radians (CW for y-down convention).
    pub rotation: f32,
    /// Per-axis scale factors.
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// Identity transform (no translation, no rotation, scale 1).
    pub const IDENTITY: Self = Self {
        translation: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    };

    /// Pure translation.
    #[must_use]
    pub const fn from_translation(translation: Vec2) -> Self {
        Self {
            translation,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Pure rotation around the origin (radians).
    #[must_use]
    pub const fn from_rotation(rotation: f32) -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation,
            scale: Vec2::ONE,
        }
    }

    /// Uniform scale.
    #[must_use]
    pub const fn from_scale(scale: Vec2) -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale,
        }
    }

    /// Apply this transform to a point.
    #[must_use]
    pub fn transform_point(&self, p: Vec2) -> Vec2 {
        let (sin, cos) = self.rotation.sin_cos();
        let scaled = Vec2::new(p.x * self.scale.x, p.y * self.scale.y);
        let rotated = Vec2::new(
            scaled.x * cos - scaled.y * sin,
            scaled.x * sin + scaled.y * cos,
        );
        rotated + self.translation
    }

    /// Apply this transform to a vector (translation is **not** applied,
    /// which is the convention for direction vectors).
    #[must_use]
    pub fn transform_vector(&self, v: Vec2) -> Vec2 {
        let (sin, cos) = self.rotation.sin_cos();
        let scaled = Vec2::new(v.x * self.scale.x, v.y * self.scale.y);
        Vec2::new(
            scaled.x * cos - scaled.y * sin,
            scaled.x * sin + scaled.y * cos,
        )
    }

    /// Return a matrix suitable for shader uniforms.
    ///
    /// The layout matches a column-major 3×3, packed into 6 floats
    /// `[ax, ay, bx, by, tx, ty]` representing the affine 2D transform:
    ///
    /// ```text
    /// | ax  bx  tx |
    /// | ay  by  ty |
    /// | 0   0   1  |
    /// ```
    #[must_use]
    pub fn to_matrix(self) -> [f32; 6] {
        let (sin, cos) = self.rotation.sin_cos();
        [
            cos * self.scale.x,
            sin * self.scale.x,
            -sin * self.scale.y,
            cos * self.scale.y,
            self.translation.x,
            self.translation.y,
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;

    fn approx_eq(a: Vec2, b: Vec2, eps: f32) -> bool {
        (a.x - b.x).abs() < eps && (a.y - b.y).abs() < eps
    }

    #[test]
    fn identity_is_neutral() {
        let p = Vec2::new(2.5, -3.7);
        assert_eq!(Transform::IDENTITY.transform_point(p), p);
    }

    #[test]
    fn pure_translation() {
        let t = Transform::from_translation(Vec2::new(1.0, 2.0));
        assert_eq!(t.transform_point(Vec2::ZERO), Vec2::new(1.0, 2.0));
    }

    #[test]
    fn rotation_90deg() {
        let t = Transform::from_rotation(FRAC_PI_2);
        // (1, 0) rotated 90° CW in y-down → (0, 1).
        let result = t.transform_point(Vec2::new(1.0, 0.0));
        assert!(approx_eq(result, Vec2::new(0.0, 1.0), 1e-5));
    }

    #[test]
    fn scale_doubles() {
        let t = Transform::from_scale(Vec2::splat(2.0));
        assert_eq!(t.transform_point(Vec2::new(3.0, 4.0)), Vec2::new(6.0, 8.0));
    }
}
