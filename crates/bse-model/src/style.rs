//! Visual style descriptors (color, stroke, fill...).

use bse_types::Color;
use serde::{Deserialize, Serialize};

/// Style applied to a shape (rectangle, ellipse, polygon).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeStyle {
    /// Stroke (outline) color, or `None` for no outline.
    pub stroke: Option<Color>,
    /// Stroke width in world units.
    pub stroke_width: f32,
    /// Fill color, or `None` for transparent fill.
    pub fill: Option<Color>,
    /// Corner radius (rectangles only) in world units.
    pub corner_radius: f32,
    /// Opacity in `0.0..=1.0`. Multiplies stroke and fill alpha.
    pub opacity: f32,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            stroke: Some(Color::BLACK),
            stroke_width: 2.0,
            fill: None,
            corner_radius: 0.0,
            opacity: 1.0,
        }
    }
}

/// Style applied to a free-hand pen stroke.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PenStyle {
    /// Pen color.
    pub color: Color,
    /// Base stroke width in world units.
    pub size: f32,
    /// Effect of pen pressure on width, in `-1.0..=1.0`.
    pub thinning: f32,
    /// Smoothing of corners in `0.0..=1.0`.
    pub smoothing: f32,
    /// Taper amount at the start of the stroke, in `0.0..=1.0`.
    pub start_taper: f32,
    /// Taper amount at the end of the stroke, in `0.0..=1.0`.
    pub end_taper: f32,
}

impl Default for PenStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            size: 4.0,
            thinning: 0.5,
            smoothing: 0.5,
            start_taper: 0.0,
            end_taper: 0.4,
        }
    }
}

/// Tagged union of styles, used when an element's style varies by kind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Style {
    /// Style for shapes (rectangle, ellipse...).
    Shape(ShapeStyle),
    /// Style for pen strokes.
    Pen(PenStyle),
}

impl Style {
    /// Default shape style.
    #[must_use]
    pub fn shape() -> Self {
        Self::Shape(ShapeStyle::default())
    }

    /// Default pen style.
    #[must_use]
    pub fn pen() -> Self {
        Self::Pen(PenStyle::default())
    }
}
