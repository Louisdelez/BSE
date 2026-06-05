//! 2D geometry primitives : `Vec2`, `Rect`, `Transform`.
//!
//! These types wrap or extend `glam` and add :
//!
//! - `serde` support out of the box,
//! - methods specialized to BSE's "y-down" canvas convention,
//! - convenience constructors.

mod rect;
mod transform;
mod vec2;

pub use rect::Rect;
pub use transform::Transform;
pub use vec2::Vec2;
