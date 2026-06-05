//! Render scene elements and the in-progress tool preview via the
//! `egui::Painter`. v005 introduced shape rendering ; v006 adds pen
//! stroke rendering using `bse-pen`.

use bse_canvas::{CanvasState, ToolKind, ToolState};
use bse_model::{
    Camera, Element, ElementKind, PenStyle as ModelPenStyle, Scene, ShapeStyle, Style, TextStyle,
    element::StrokePoint,
};
use bse_pen::{InputPoint, StrokeOptions, get_stroke};
use bse_spatial::Quadtree;
use bse_types::{Color, ElementId, PeerId, Rect as WorldRect, Transform, Vec2 as WorldVec2};
use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke};

/// Render every element of `scene` that intersects the camera viewport,
/// in z-order. Returns the count of elements actually drawn.
///
/// `spatial` is used for viewport culling : without it, large scenes
/// (~10 000 elements) would force every element through the renderer
/// every frame.
pub fn elements(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    scene: &Scene,
    spatial: &Quadtree<ElementId>,
) -> u32 {
    let world_viewport = camera.viewport_world_rect(viewport);
    let visible_ids = spatial.query(world_viewport);
    let mut visible: Vec<&Element> = visible_ids
        .into_iter()
        .filter_map(|id| scene.get(id))
        .collect();
    visible.sort_by(|a, b| a.z.cmp(&b.z).then(a.id.as_uuid().cmp(&b.id.as_uuid())));
    for element in &visible {
        paint_element(painter, rect, camera, viewport, element);
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
        ToolState::Idle => {}
        ToolState::DrawingShape {
            anchor_world,
            current_world,
        } => {
            if let Some(element) = preview_shape(canvas.tool, *anchor_world, *current_world) {
                paint_element(painter, rect, camera, viewport, &element);
            }
        }
        ToolState::DrawingStroke { points } => {
            paint_stroke_outline(painter, rect, camera, viewport, points, preview_pen_color());
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

/// Build a final pen `Element` from accumulated stroke samples. Returns
/// `None` if the input is empty.
#[must_use]
pub fn commit_stroke(points: &[InputPoint]) -> Option<Element> {
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
        style: Style::Pen(ModelPenStyle::default()),
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

fn preview_pen_color() -> Color {
    Color::rgba(0x42, 0x62, 0xFF, 0xA0)
}

fn paint_element(
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
        (ElementKind::Pen { points }, Style::Pen(s)) => {
            let inputs: Vec<InputPoint> = points
                .iter()
                .map(|p| InputPoint::new(p.x, p.y, p.pressure))
                .collect();
            paint_stroke_outline(painter, rect, camera, viewport, &inputs, s.color);
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

fn paint_stroke_outline(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    points: &[InputPoint],
    color: Color,
) {
    if points.len() < 2 {
        return;
    }
    let outline = get_stroke(points, &StrokeOptions::default());
    if outline.len() < 3 {
        return;
    }
    let screen_points: Vec<Pos2> = outline
        .into_iter()
        .map(|p| world_to_screen(camera, viewport, rect, p))
        .collect();
    let fill = to_color32(color, 1.0);
    painter.add(egui::Shape::convex_polygon(
        screen_points,
        fill,
        Stroke::NONE,
    ));
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
