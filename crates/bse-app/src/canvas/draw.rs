//! Render scene elements and the in-progress tool preview via the
//! `egui::Painter`. v005 introduced shape rendering ; v006 adds pen
//! stroke rendering ; v013 adds text ; v014 adds raster images.

use bse_canvas::{CanvasState, ToolKind, ToolState};
use bse_crdt::{CrdtBackend, YrsBackend};
use bse_model::{
    Camera, Element, ElementKind, ImageStyle, PenStyle as ModelPenStyle, ShapeStyle, Style,
    TextStyle, element::StrokePoint,
};
use bse_pen::{InputPoint, StrokeOptions, get_stroke};
use bse_spatial::Quadtree;
use bse_types::{
    AssetId, Color, ElementId, PeerId, Rect as WorldRect, Transform, Vec2 as WorldVec2,
};
use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke};

use crate::assets::AssetStore;

/// Render every visible element of the CRDT-backed document, in z-order.
/// Returns the count of elements actually drawn.
#[allow(clippy::too_many_arguments)]
pub fn elements(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    crdt: &YrsBackend,
    spatial: &Quadtree<ElementId>,
    assets: &mut AssetStore,
    ctx: &egui::Context,
) -> u32 {
    let world_viewport = camera.viewport_world_rect(viewport);
    let visible_ids = spatial.query(world_viewport);
    let mut visible: Vec<Element> = visible_ids
        .into_iter()
        .filter_map(|id| crdt.get_element(id))
        .collect();
    visible.sort_by(|a, b| a.z.cmp(&b.z).then(a.id.as_uuid().cmp(&b.id.as_uuid())));
    for element in &visible {
        paint_element(painter, rect, camera, viewport, element, assets, ctx);
    }
    u32::try_from(visible.len()).unwrap_or(u32::MAX)
}

/// Render the in-progress tool preview, if any.
pub fn tool_preview(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    canvas: &CanvasState,
) {
    match &canvas.tool_state {
        ToolState::Idle | ToolState::EditingText { .. } => {}
        ToolState::DrawingShape {
            anchor_world,
            current_world,
        } => {
            if let Some(element) = preview_shape(canvas.tool, *anchor_world, *current_world) {
                paint_shape_only(painter, rect, camera, viewport, &element);
            }
        }
        ToolState::DrawingStroke { points } => {
            // Match the screen-space sizing used at commit time
            // (see `commit_stroke` invocation in canvas::input). The
            // preview uses world-space size = chosen_size / zoom so
            // the drag visual matches the final stored element.
            let zoom = canvas.camera.zoom.max(0.0001);
            paint_stroke_outline(
                painter,
                rect,
                camera,
                viewport,
                points,
                canvas.pen_style.color,
                canvas.pen_style.size / zoom,
            );
        }
    }
}

/// Paint every remote peer's cursor in world space.
pub fn remote_cursors(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    peers: &crate::peers::PeerStore,
) {
    for (_, peer) in peers.with_cursors() {
        let Some(world) = peer.last_cursor else {
            continue;
        };
        let pos = world_to_screen(camera, viewport, rect, world);
        if !rect.expand(40.0).contains(pos) {
            continue;
        }
        let color = to_color32(peer.color, 1.0);
        let p0 = pos;
        let p1 = Pos2::new(pos.x, pos.y + 14.0);
        let p2 = Pos2::new(pos.x + 10.0, pos.y + 10.0);
        painter.add(egui::Shape::convex_polygon(
            vec![p0, p1, p2],
            color,
            Stroke::new(1.0, Color32::WHITE),
        ));
        if let Some(name) = peer.display_name.as_deref() {
            let label_pos = Pos2::new(pos.x + 12.0, pos.y + 12.0);
            painter.text(
                label_pos,
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::proportional(11.0),
                color,
            );
        }
    }
}

/// Build a final shape `Element` from a finished drag. Returns `None`
/// for degenerate drags (less than 1 unit on either axis).
#[must_use]
pub fn commit_shape(tool: ToolKind, anchor: WorldVec2, current: WorldVec2) -> Option<Element> {
    let world_rect = WorldRect::from_two_points(anchor, current);
    let size = world_rect.size();
    if size.x.abs() < 1.0 || size.y.abs() < 1.0 {
        return None;
    }
    shape_element(tool, world_rect, default_shape_style())
}

/// Build a `Text` element at `position` with a placeholder content.
///
/// v013 spawns a `"Text"` placeholder ; inline editing comes in a later
/// milestone.
#[must_use]
pub fn commit_text(position: WorldVec2) -> Element {
    Element {
        id: ElementId::new_v7(),
        kind: ElementKind::Text {
            content: "Text".to_string(),
            font_size: 16.0,
        },
        style: Style::Text(TextStyle::default()),
        transform: Transform::from_translation(position),
        z: 0,
        created_by: PeerId::default(),
        created_at: 0,
    }
}

/// Build an `Image` element at the given position, sized so it fits a
/// sensible default world-space rectangle (max 400 unit wide / tall,
/// preserving aspect ratio).
#[must_use]
pub fn commit_image(position: WorldVec2, asset_id: AssetId, px_w: u32, px_h: u32) -> Element {
    const MAX: f32 = 400.0;
    // Cast note : decoded image dimensions fit in `f32` mantissa for any
    // realistic file (< ~16M × 16M).
    #[allow(clippy::cast_precision_loss)]
    let original_w = px_w as f32;
    #[allow(clippy::cast_precision_loss)]
    let original_h = px_h as f32;
    let aspect = original_w / original_h.max(1.0);
    let (w, h) = if original_w <= MAX && original_h <= MAX {
        (original_w, original_h)
    } else if aspect >= 1.0 {
        (MAX, MAX / aspect)
    } else {
        (MAX * aspect, MAX)
    };
    Element {
        id: ElementId::new_v7(),
        kind: ElementKind::Image {
            asset_id,
            width: w,
            height: h,
        },
        style: Style::Image(ImageStyle::default()),
        transform: Transform::from_translation(position),
        z: 0,
        created_by: PeerId::default(),
        created_at: 0,
    }
}

/// Build a final pen `Element` from accumulated stroke samples. Returns
/// `None` if the input is empty.
#[must_use]
pub fn commit_stroke(points: &[InputPoint], style: ModelPenStyle) -> Option<Element> {
    if points.is_empty() {
        return None;
    }
    Some(Element {
        id: ElementId::new_v7(),
        kind: ElementKind::Pen {
            points: points
                .iter()
                .map(|p| StrokePoint {
                    x: p.x,
                    y: p.y,
                    pressure: p.pressure,
                })
                .collect(),
        },
        style: Style::Pen(style),
        transform: Transform::IDENTITY,
        z: 0,
        created_by: PeerId::default(),
        created_at: 0,
    })
}

fn preview_shape(tool: ToolKind, anchor: WorldVec2, current: WorldVec2) -> Option<Element> {
    let world_rect = WorldRect::from_two_points(anchor, current);
    shape_element(tool, world_rect, preview_shape_style())
}

fn shape_element(tool: ToolKind, world_rect: WorldRect, style: ShapeStyle) -> Option<Element> {
    let kind = match tool {
        ToolKind::Rectangle => ElementKind::Rectangle {
            width: world_rect.width(),
            height: world_rect.height(),
        },
        ToolKind::Ellipse => ElementKind::Ellipse {
            width: world_rect.width(),
            height: world_rect.height(),
        },
        ToolKind::Line => ElementKind::Line {
            end: world_rect.max - world_rect.min,
        },
        _ => return None,
    };
    Some(Element {
        id: ElementId::new_v7(),
        kind,
        style: Style::Shape(style),
        transform: Transform::from_translation(world_rect.center()),
        z: 0,
        created_by: PeerId::default(),
        created_at: 0,
    })
}

fn default_shape_style() -> ShapeStyle {
    ShapeStyle {
        stroke: Some(Color::rgb(0x1C, 0x1C, 0x1E)),
        stroke_width: 2.0,
        fill: Some(Color::rgb(0xFF, 0xD0, 0x2F)),
        corner_radius: 4.0,
        opacity: 1.0,
    }
}

fn preview_shape_style() -> ShapeStyle {
    ShapeStyle {
        stroke: Some(Color::rgb(0x42, 0x62, 0xFF)),
        stroke_width: 1.5,
        fill: None,
        corner_radius: 4.0,
        opacity: 0.6,
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_element(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    element: &Element,
    assets: &mut AssetStore,
    ctx: &egui::Context,
) {
    let translation = element.transform.translation;
    match (&element.kind, &element.style) {
        (ElementKind::Rectangle { width, height }, Style::Shape(s)) => {
            paint_rectangle(
                painter,
                rect,
                camera,
                viewport,
                translation,
                *width,
                *height,
                s,
            );
        }
        (ElementKind::Ellipse { width, height }, Style::Shape(s)) => {
            paint_ellipse(
                painter,
                rect,
                camera,
                viewport,
                translation,
                *width,
                *height,
                s,
            );
        }
        (ElementKind::Line { end }, Style::Shape(s)) => {
            paint_line(painter, rect, camera, viewport, translation, *end, s);
        }
        (ElementKind::Pen { points }, Style::Pen(s)) => {
            let inputs: Vec<InputPoint> = points
                .iter()
                .map(|p| InputPoint::new(p.x, p.y, p.pressure))
                .collect();
            paint_stroke_outline(painter, rect, camera, viewport, &inputs, s.color, s.size);
        }
        (ElementKind::Text { content, font_size }, Style::Text(s)) => {
            paint_text(
                painter,
                rect,
                camera,
                viewport,
                translation,
                content,
                *font_size,
                s,
            );
        }
        (
            ElementKind::Image {
                asset_id,
                width,
                height,
            },
            Style::Image(s),
        ) => {
            paint_image(
                painter,
                rect,
                camera,
                viewport,
                translation,
                *width,
                *height,
                *asset_id,
                s,
                assets,
                ctx,
            );
        }
        _ => {}
    }
}

/// Paint a non-image element. Used by `tool_preview` which has no
/// `AssetStore` access (previews are never images).
fn paint_shape_only(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    element: &Element,
) {
    let translation = element.transform.translation;
    match (&element.kind, &element.style) {
        (ElementKind::Rectangle { width, height }, Style::Shape(s)) => {
            paint_rectangle(
                painter,
                rect,
                camera,
                viewport,
                translation,
                *width,
                *height,
                s,
            );
        }
        (ElementKind::Ellipse { width, height }, Style::Shape(s)) => {
            paint_ellipse(
                painter,
                rect,
                camera,
                viewport,
                translation,
                *width,
                *height,
                s,
            );
        }
        (ElementKind::Line { end }, Style::Shape(s)) => {
            paint_line(painter, rect, camera, viewport, translation, *end, s);
        }
        _ => {}
    }
}

fn paint_rectangle(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    center: WorldVec2,
    width: f32,
    height: f32,
    style: &ShapeStyle,
) {
    let half = WorldVec2::new(width, height) / 2.0;
    let min = world_to_screen(camera, viewport, rect, center - half);
    let max = world_to_screen(camera, viewport, rect, center + half);
    let r = Rect::from_min_max(min, max);
    let radius = Rounding::same(style.corner_radius * camera.zoom);
    if let Some(fill) = style.fill {
        painter.rect_filled(r, radius, to_color32(fill, style.opacity));
    }
    if let Some(stroke_color) = style.stroke {
        let stroke = Stroke::new(
            style.stroke_width * camera.zoom,
            to_color32(stroke_color, style.opacity),
        );
        painter.rect_stroke(r, radius, stroke);
    }
}

fn paint_ellipse(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    center: WorldVec2,
    width: f32,
    height: f32,
    style: &ShapeStyle,
) {
    let screen_center = world_to_screen(camera, viewport, rect, center);
    let radius = egui::vec2(width * camera.zoom * 0.5, height * camera.zoom * 0.5);
    if let Some(fill) = style.fill {
        painter.add(egui::Shape::ellipse_filled(
            screen_center,
            radius,
            to_color32(fill, style.opacity),
        ));
    }
    if let Some(stroke_color) = style.stroke {
        painter.add(egui::Shape::ellipse_stroke(
            screen_center,
            radius,
            Stroke::new(
                style.stroke_width * camera.zoom,
                to_color32(stroke_color, style.opacity),
            ),
        ));
    }
}

fn paint_line(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    start: WorldVec2,
    end: WorldVec2,
    style: &ShapeStyle,
) {
    let Some(stroke_color) = style.stroke else {
        return;
    };
    let p0 = world_to_screen(camera, viewport, rect, start);
    let p1 = world_to_screen(camera, viewport, rect, start + end);
    painter.line_segment(
        [p0, p1],
        Stroke::new(
            style.stroke_width * camera.zoom,
            to_color32(stroke_color, style.opacity),
        ),
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_image(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    center: WorldVec2,
    width: f32,
    height: f32,
    asset_id: AssetId,
    style: &ImageStyle,
    assets: &mut AssetStore,
    ctx: &egui::Context,
) {
    let Some(texture) = assets.texture(asset_id, ctx) else {
        // Asset missing — render a placeholder so the user sees that
        // *something* should be there.
        let half = WorldVec2::new(width, height) / 2.0;
        let min = world_to_screen(camera, viewport, rect, center - half);
        let max = world_to_screen(camera, viewport, rect, center + half);
        painter.rect_filled(
            Rect::from_min_max(min, max),
            Rounding::same(2.0),
            Color32::from_rgba_unmultiplied(0xC7, 0xCA, 0xD5, 0x80),
        );
        return;
    };
    let half = WorldVec2::new(width, height) / 2.0;
    let min = world_to_screen(camera, viewport, rect, center - half);
    let max = world_to_screen(camera, viewport, rect, center + half);
    let screen_rect = Rect::from_min_max(min, max);
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    // Cast note : the clamp keeps the value in [0, 255], so the truncation
    // is exact.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let alpha = (style.opacity * 255.0).clamp(0.0, 255.0) as u8;
    let tint = Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, alpha);
    painter.image(texture.id(), screen_rect, uv, tint);
}

#[allow(clippy::too_many_arguments)]
fn paint_text(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    center: WorldVec2,
    content: &str,
    font_size: f32,
    style: &TextStyle,
) {
    let screen_center = world_to_screen(camera, viewport, rect, center);
    let screen_font_size = (font_size * camera.zoom).clamp(2.0, 256.0);
    painter.text(
        screen_center,
        egui::Align2::CENTER_CENTER,
        content,
        egui::FontId::proportional(screen_font_size),
        to_color32(style.color, 1.0),
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_stroke_outline(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    points: &[InputPoint],
    color: Color,
    size: f32,
) {
    if points.len() < 2 {
        return;
    }
    let options = StrokeOptions {
        size,
        ..StrokeOptions::default()
    };
    let outline = get_stroke(points, &options);
    if outline.len() < 3 {
        return;
    }
    let screen_points: Vec<Pos2> = outline
        .into_iter()
        .map(|p| world_to_screen(camera, viewport, rect, p))
        .collect();
    paint_concave_polygon(painter, &screen_points, to_color32(color, 1.0));
}

/// Triangulate a (possibly concave, self-non-intersecting) polygon
/// outline via ear-clipping and submit it as an `egui::Mesh`.
///
/// Pen-stroke outlines from `perfect-freehand` are long snake-like
/// shapes that `Shape::convex_polygon` cannot handle — it fan-
/// triangulates from the first vertex, producing the giant
/// triangular blobs that v030 surfaced. `earcutr` handles concave
/// polygons correctly.
fn paint_concave_polygon(painter: &egui::Painter, screen_points: &[Pos2], color: Color32) {
    if screen_points.len() < 3 {
        return;
    }
    // earcutr takes a flat `Vec<f64>` (xy interleaved) and returns
    // triangle indices.
    let flat: Vec<f64> = screen_points
        .iter()
        .flat_map(|p| [f64::from(p.x), f64::from(p.y)])
        .collect();
    let Ok(indices) = earcutr::earcut(&flat, &[], 2) else {
        // Earcut failed — fall back to the (broken) convex variant
        // rather than dropping the stroke entirely.
        painter.add(egui::Shape::convex_polygon(
            screen_points.to_vec(),
            color,
            Stroke::NONE,
        ));
        return;
    };
    if indices.is_empty() {
        return;
    }
    let mut mesh = egui::epaint::Mesh::default();
    for p in screen_points {
        mesh.colored_vertex(*p, color);
    }
    for tri in indices.chunks_exact(3) {
        let a = u32::try_from(tri[0]).unwrap_or(0);
        let b = u32::try_from(tri[1]).unwrap_or(0);
        let c = u32::try_from(tri[2]).unwrap_or(0);
        mesh.add_triangle(a, b, c);
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn world_to_screen(camera: &Camera, viewport: WorldVec2, rect: Rect, world: WorldVec2) -> Pos2 {
    let screen = camera.world_to_screen(viewport, world);
    Pos2::new(rect.min.x + screen.x, rect.min.y + screen.y)
}

fn to_color32(color: Color, opacity: f32) -> Color32 {
    // Cast note : the clamp keeps the value in [0, 255], so the truncation
    // to `u8` is exact.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let alpha = (f32::from(color.a) * opacity).clamp(0.0, 255.0) as u8;
    Color32::from_rgba_unmultiplied(color.r, color.g, color.b, alpha)
}
