//! Foundational types shared across all BSE crates.
//!
//! This crate has no dependency on any other BSE crate. It defines :
//!
//! - [`ids`] : strongly-typed identifiers (project, element, user, peer, asset).
//! - [`geometry`] : 2D math primitives (`Vec2`, `Rect`, `Transform`).
//! - [`color`] : RGBA color representation.
//!
//! Keeping these types in a dedicated crate prevents circular dependencies
//! and provides a stable foundation that rarely changes.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod color;
pub mod geometry;
pub mod ids;

pub use color::Color;
pub use geometry::{Rect, Transform, Vec2};
pub use ids::{AssetId, CommentId, ElementId, PeerId, ProjectId, UserId};
