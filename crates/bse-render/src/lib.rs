//! GPU renderer for the BSE canvas.
//!
//! The renderer turns a [`Scene`](bse_model::Scene) into pixels on a
//! `wgpu::TextureView`. It is intentionally **stateless across frames** :
//! each [`render`](Renderer::render) call rebuilds the GPU buffers from
//! the visible subset of the scene.
//!
//! This v002 milestone only exposes the [`Renderer`] trait and a
//! placeholder implementation that records what *would* be rendered.
//! The real wgpu pipelines (`background`, `shapes`, `strokes`) are
//! introduced in v003 and v005.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod renderer;
mod stats;

pub use renderer::{NullRenderer, Renderer};
pub use stats::FrameStats;
