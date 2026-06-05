//! Tool selection enum.

use serde::{Deserialize, Serialize};

/// The kind of tool currently active on the canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    /// Select / move / resize existing elements.
    #[default]
    Select,
    /// Free-hand pen drawing.
    Pen,
    /// Create a rectangle by click-and-drag.
    Rectangle,
    /// Create an ellipse by click-and-drag.
    Ellipse,
    /// Create a line.
    Line,
    /// Create a text block.
    Text,
}

impl ToolKind {
    /// Single-character display label, useful for compact toolbars.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Pen => "Pen",
            Self::Rectangle => "Rect",
            Self::Ellipse => "Ellipse",
            Self::Line => "Line",
            Self::Text => "Text",
        }
    }
}
