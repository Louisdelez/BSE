//! Render scene elements and the in-progress tool preview via the
//! `egui::Painter`. v005 uses the painter rather than a `wgpu` callback ;
//! a real GPU pipeline will replace this in v009+.

use bse_canvas::{CanvasState, ToolKind, ToolState};
use bse_model::{Camera, Element, ElementKind, Scene, ShapeStyle, Style};
use bse_types::{Color, ElementId, PeerId, Rect as WorldRect, Transform, Vec2 as WorldVec2};
use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke};

/// Render every element of `scene` in z-order.
pub fn elements(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    scene: &Scene,
) {
    for element in scene.iter_z_sorted() {
        paint_element(painter, rect, camera, viewport, element);
    }
}

/// Render the in-progress tool preview, if any.
pub fn tool_preview(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    canvas: &CanvasState,
) {
    if let ToolState::DrawingShape {
        anchor_world,
        current_world,
    } = canvas.tool_state
        && let Some(element) = preview_element(canvas.tool, anchor_world, current_world)
    {
        paint_element(painter, rect, camera, viewport, &element);
    }
}

/// Build an [`Element`] from a finished drag. Returns `None` if the
/// drag was degenerate (less than 1 px world span on either axis).
#[must_use]
pub fn commit_shape(tool: ToolKind, anchor: WorldVec2, current: WorldVec2) -> Option<Element> {
    let world_rect = WorldRect::from_two_points(anchor, current);
    let size = world_rect.size();
    if size.x.abs() < 1.0 || size.y.abs() < 1.0 {
        return None;
    }
    element_for(tool, world_rect, default_style())
}

fn preview_element(tool: ToolKind, anchor: WorldVec2, current: WorldVec2) -> Option<Element> {
    let world_rect = WorldRect::from_two_points(anchor, current);
    element_for(tool, world_rect, preview_style())
}

fn element_for(tool: ToolKind, world_rect: WorldRect, style: ShapeStyle) -> Option<Element> {
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
    let center = world_rect.center();
    Some(Element {
        id: ElementId::new_v7(),
        kind,
        style: Style::Shape(style),
        transform: Transform::from_translation(center),
        z: 0,
        created_by: PeerId::default(),
        created_at: 0,
    })
}

fn default_style() -> ShapeStyle {
    ShapeStyle {
        stroke: Some(Color::rgb(0x1C, 0x1C, 0x1E)),
        stroke_width: 2.0,
        fill: Some(Color::rgb(0xFF, 0xD0, 0x2F)),
        corner_radius: 4.0,
        opacity: 1.0,
    }
}

fn preview_style() -> ShapeStyle {
    ShapeStyle {
        stroke: Some(Color::rgb(0x42, 0x62, 0xFF)),
        stroke_width: 1.5,
        fill: None,
        corner_radius: 4.0,
        opacity: 0.6,
    }
}

fn paint_element(
    painter: &egui::Painter,
    rect: Rect,
    camera: &Camera,
    viewport: WorldVec2,
    element: &Element,
) {
    let Style::Shape(style) = &element.style else {
        return; // Non-shape styles land in later milestones.
    };
    let translation = element.transform.translation;
    match element.kind {
        ElementKind::Rectangle { width, height } => paint_rectangle(
            painter,
            rect,
            camera,
            viewport,
            translation,
            width,
            height,
            style,
        ),
        ElementKind::Ellipse { width, height } => {
            paint_ellipse(painter, rect, camera, viewport, translation, width, height, style);
        }
        ElementKind::Line { end } => {
            paint_line(painter, rect, camera, viewport, translation, end, style);
        }
        ElementKind::Pen { .. } => { /* Pen lands in v006. */ }
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
    if let Some(fill) = style.fill {
        painter.rect_filled(r, Rounding::same(style.corner_radius * camera.zoom), to_color32(fill, style.opacity));
    }
    if let Some(stroke_color) = style.stroke {
        let stroke = Stroke::new(style.stroke_width * camera.zoom, to_color32(stroke_color, style.opacity));
        painter.rect_stroke(r, Rounding::same(style.corner_radius * camera.zoom), stroke);
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
            Stroke::new(style.stroke_width * camera.zoom, to_color32(stroke_color, style.opacity)),
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
    let Some(stroke_color) = style.stroke else { return };
    let p0 = world_to_screen(camera, viewport, rect, start);
    let p1 = world_to_screen(camera, viewport, rect, start + end);
    painter.line_segment(
        [p0, p1],
        Stroke::new(style.stroke_width * camera.zoom, to_color32(stroke_color, style.opacity)),
    );
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
