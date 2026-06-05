//! Mutable state owned by the canvas.

use bse_model::Camera;
use bse_pen::InputPoint;
use bse_types::Vec2;

use crate::tool::ToolKind;

/// Top-level canvas state.
#[derive(Clone, Debug, Default)]
pub struct CanvasState {
    /// Active camera (per-peer, not in the CRDT).
    pub camera: Camera,
    /// Currently active tool.
    pub tool: ToolKind,
    /// Transient state for a tool that has a multi-frame interaction.
    pub tool_state: ToolState,
}

impl CanvasState {
    /// New canvas centered at origin, zoom 1, Select tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Switch the active tool. Any in-progress tool interaction is dropped.
    pub fn set_tool(&mut self, tool: ToolKind) {
        if self.tool != tool {
            self.tool_state = ToolState::Idle;
        }
        self.tool = tool;
    }
}

/// Per-frame state for tools that span multiple frames (drag-to-draw, etc.).
///
/// Reset to [`ToolState::Idle`] between interactions and whenever the active
/// tool changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ToolState {
    /// No interaction in progress.
    #[default]
    Idle,
    /// User is drag-creating a shape. The anchor is in world coordinates
    /// and is the first corner of the shape.
    DrawingShape {
        /// Anchor (start point) in world space.
        anchor_world: Vec2,
        /// Current pointer position in world space (updated each frame).
        current_world: Vec2,
    },
    /// User is drawing a free-hand pen stroke. Points are accumulated
    /// in world space at the current pressure.
    DrawingStroke {
        /// Stroke samples so far, in world space.
        points: Vec<InputPoint>,
    },
}
