//! Domain model of BSE.
//!
//! This crate defines the in-memory representation of a BSE project :
//! its elements, their style, the camera state, and the scene container.
//!
//! These types are **purely data** : no I/O, no CRDT, no rendering.
//! The CRDT layer ([`bse-crdt`]) wraps them and the renderer
//! ([`bse-render`]) consumes them.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod camera;
pub mod element;
pub mod scene;
pub mod style;

pub use camera::Camera;
pub use element::{Element, ElementKind};
pub use scene::Scene;
pub use style::{ImageStyle, PenStyle, ShapeStyle, Style, TextStyle};
