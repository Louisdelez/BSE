//! Mutable state owned by the canvas.

use bse_model::Camera;

use crate::tool::ToolKind;

/// Top-level canvas state.
#[derive(Clone, Debug, Default)]
pub struct CanvasState {
    /// Active camera (per-peer, not in the CRDT).
    pub camera: Camera,
    /// Currently active tool.
    pub tool: ToolKind,
}

impl CanvasState {
    /// New canvas centered at origin, zoom 1, Select tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
