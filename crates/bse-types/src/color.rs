//! RGBA color, 8 bits per channel.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// RGBA color with 8 bits per channel.
///
/// The byte ordering is `(r, g, b, a)`, matching `Bgra8` GPU formats
/// when reading components by index. Use [`Color::to_linear`] to
/// convert to a linear-space `[f32; 4]` for shader uniforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct Color {
    /// Red channel, 0..=255.
    pub r: u8,
    /// Green channel, 0..=255.
    pub g: u8,
    /// Blue channel, 0..=255.
    pub b: u8,
    /// Alpha channel, 0..=255. `255` is fully opaque.
    pub a: u8,
}

impl Color {
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    /// Pure white, fully opaque.
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// Pure black, fully opaque.
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    /// BSE brand yellow (from the Miro-inspired design system).
    pub const BRAND_YELLOW: Self = Self::rgb(0xFF, 0xD0, 0x2F);
    /// BSE brand blue.
    pub const BRAND_BLUE: Self = Self::rgb(0x42, 0x62, 0xFF);

    /// Construct an opaque color from RGB components.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Construct a color from RGBA components.
    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Construct from a packed 0xRRGGBBAA `u32`.
    #[must_use]
    pub const fn from_rgba_u32(rgba: u32) -> Self {
        Self {
            r: ((rgba >> 24) & 0xFF) as u8,
            g: ((rgba >> 16) & 0xFF) as u8,
            b: ((rgba >> 8) & 0xFF) as u8,
            a: (rgba & 0xFF) as u8,
        }
    }

    /// Pack into a `0xRRGGBBAA` `u32`.
    #[must_use]
    pub const fn to_rgba_u32(self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    /// Convert to linear-space `[f32; 4]` suitable for shader uniforms.
    ///
    /// The conversion treats the input as sRGB and applies the standard
    /// gamma curve. Alpha is passed through linearly.
    #[must_use]
    pub fn to_linear(self) -> [f32; 4] {
        [
            srgb_to_linear(f32::from(self.r) / 255.0),
            srgb_to_linear(f32::from(self.g) / 255.0),
            srgb_to_linear(f32::from(self.b) / 255.0),
            f32::from(self.a) / 255.0,
        ]
    }

    /// Convert to a naive (non-gamma-corrected) `[f32; 4]`.
    ///
    /// Use this for UI elements where you want exact color matching.
    #[must_use]
    pub fn to_f32(self) -> [f32; 4] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            f32::from(self.a) / 255.0,
        ]
    }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brand_yellow_packed() {
        let c = Color::BRAND_YELLOW;
        assert_eq!(c.to_rgba_u32(), 0xFFD0_2FFF);
    }

    #[test]
    fn from_to_u32_roundtrip() {
        let c = Color::from_rgba_u32(0x1234_5678);
        assert_eq!(c.to_rgba_u32(), 0x1234_5678);
    }

    #[test]
    fn white_linear() {
        let lin = Color::WHITE.to_linear();
        assert!((lin[0] - 1.0).abs() < 1e-4);
        assert!((lin[3] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn black_linear_is_zero() {
        let lin = Color::BLACK.to_linear();
        assert!(lin[0].abs() < f32::EPSILON);
        assert!((lin[3] - 1.0).abs() < f32::EPSILON);
    }
}
