//! Element : the atomic unit of content on the canvas.

use bse_types::{ElementId, PeerId, Rect, Transform, Vec2};
use serde::{Deserialize, Serialize};

use crate::style::Style;

/// An element on the canvas (a shape, a pen stroke, a text block, ...).
///
/// Every visible object on the toile is an `Element`. The element's
/// **identity** is its `id` ; its **geometry** is encoded in `transform`
/// and in the kind-specific fields ; its **appearance** is the `style`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Stable identifier.
    pub id: ElementId,
    /// What kind of element this is.
    pub kind: ElementKind,
    /// Style applied at render time.
    pub style: Style,
    /// 2D affine transform (position, rotation, scale).
    pub transform: Transform,
    /// Z-order : higher draws on top.
    pub z: i32,
    /// Peer who created the element (for awareness display).
    pub created_by: PeerId,
    /// Creation timestamp, ms since UNIX epoch.
    pub created_at: u64,
}

impl Element {
    /// Compute the axis-aligned bounding box of the element, in world space.
    ///
    /// This ignores rotation — the AABB of a rotated shape is its
    /// bounding box without rotation, which is the most useful for
    /// spatial indexing and culling.
    #[must_use]
    pub fn aabb(&self) -> Rect {
        let local = self.kind.local_bbox();
        let min = self.transform.transform_point(local.min);
        let max = self.transform.transform_point(local.max);
        Rect::from_two_points(min, max)
    }
}

/// Tagged union of element kinds.
///
/// Each variant carries the kind-specific geometry. Common attributes
/// (transform, style, id) live on the parent [`Element`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ElementKind {
    /// Axis-aligned rectangle.
    Rectangle {
        /// Width in world units.
        width: f32,
        /// Height in world units.
        height: f32,
    },
    /// Axis-aligned ellipse.
    Ellipse {
        /// Width in world units.
        width: f32,
        /// Height in world units.
        height: f32,
    },
    /// Straight line from origin to `end` (local space).
    Line {
        /// Endpoint in local space (origin is `(0, 0)`).
        end: Vec2,
    },
    /// Free-hand pen stroke.
    Pen {
        /// Sample points along the stroke (in local space).
        points: Vec<StrokePoint>,
    },
    /// Single-line or wrapped text block.
    Text {
        /// Text content. Plain string for v013 ; rich-text comes later.
        content: String,
        /// Font size in world units. Default `16.0`.
        font_size: f32,
    },
    /// Raster image. Bytes live in the app's [`AssetStore`] (not in the CRDT).
    Image {
        /// Identifier of the asset blob in the asset store.
        asset_id: bse_types::AssetId,
        /// Width in world units.
        width: f32,
        /// Height in world units.
        height: f32,
    },
}

impl ElementKind {
    /// Local-space bounding box of the kind (before applying transform).
    #[must_use]
    pub fn local_bbox(&self) -> Rect {
        match self {
            Self::Rectangle { width, height }
            | Self::Ellipse { width, height }
            | Self::Image { width, height, .. } => {
                Rect::from_center_size(Vec2::ZERO, Vec2::new(*width, *height))
            }
            Self::Line { end } => Rect::from_two_points(Vec2::ZERO, *end),
            Self::Pen { points } => {
                if points.is_empty() {
                    Rect::EMPTY
                } else {
                    let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
                    let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
                    for p in points {
                        let pos = Vec2::new(p.x, p.y);
                        min = min.min(pos);
                        max = max.max(pos);
                    }
                    Rect::from_min_max(min, max)
                }
            }
            Self::Text { content, font_size } => {
                // Approximation : average glyph ~ 0.55 × font_size wide,
                // line height ~ 1.2 × font_size. Good enough for spatial
                // index ; real bbox comes from font metrics at render time.
                // Cast note : the visible line and char count of a text
                // block stays well under `u32::MAX`, so the lossy cast is
                // safe by construction.
                #[allow(clippy::cast_precision_loss)]
                let dims = {
                    let line_count = content.lines().count().max(1) as f32;
                    let max_chars = content
                        .lines()
                        .map(|l| l.chars().count())
                        .max()
                        .unwrap_or(1) as f32;
                    (max_chars, line_count)
                };
                let width = dims.0 * font_size * 0.55;
                let height = dims.1 * font_size * 1.2;
                Rect::from_center_size(Vec2::ZERO, Vec2::new(width, height))
            }
        }
    }
}

/// A single sample point along a pen stroke.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    /// X coordinate in local space.
    pub x: f32,
    /// Y coordinate in local space.
    pub y: f32,
    /// Stylus pressure in `0.0..=1.0`, or `0.5` for fallback.
    pub pressure: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rect(width: f32, height: f32) -> Element {
        Element {
            id: ElementId::new(),
            kind: ElementKind::Rectangle { width, height },
            style: Style::shape(),
            transform: Transform::IDENTITY,
            z: 0,
            created_by: PeerId::new(),
            created_at: 0,
        }
    }

    #[test]
    fn rectangle_bbox() {
        let r = make_rect(100.0, 50.0);
        let bbox = r.aabb();
        assert_eq!(bbox.size(), Vec2::new(100.0, 50.0));
    }

    #[test]
    fn empty_pen_has_empty_bbox() {
        let kind = ElementKind::Pen { points: Vec::new() };
        assert_eq!(kind.local_bbox(), Rect::EMPTY);
    }

    #[test]
    fn pen_bbox_spans_points() {
        let kind = ElementKind::Pen {
            points: vec![
                StrokePoint {
                    x: 0.0,
                    y: 0.0,
                    pressure: 0.5,
                },
                StrokePoint {
                    x: 10.0,
                    y: 0.0,
                    pressure: 0.5,
                },
                StrokePoint {
                    x: 5.0,
                    y: 20.0,
                    pressure: 0.5,
                },
            ],
        };
        let bbox = kind.local_bbox();
        assert_eq!(bbox.min, Vec2::ZERO);
        assert_eq!(bbox.max, Vec2::new(10.0, 20.0));
    }
}
