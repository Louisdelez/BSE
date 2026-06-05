//! Input types and configuration for the stroke algorithm.
//!
//! These mirror the option object of the original
//! [perfect-freehand](https://github.com/steveruizok/perfect-freehand)
//! package, simplified to a single flat struct of `f32` fields.
//!
//! TODO(v006.1): expose configurable easing functions for the body
//! and for the start/end tapers. The TypeScript original allows any
//! `(t: number) => number`; for v006 we hard-code the defaults.

/// A single sample from a pointing device.
///
/// Coordinates are in world units, with the y axis pointing **down**
/// (same convention as the rest of BSE and the original TypeScript
/// implementation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputPoint {
    /// Horizontal position.
    pub x: f32,
    /// Vertical position (positive = down).
    pub y: f32,
    /// Pressure, clamped to `0.0..=1.0`. Use `0.5` if the device does
    /// not report pressure and you do not enable
    /// [`StrokeOptions::simulate_pressure`].
    pub pressure: f32,
}

impl InputPoint {
    /// Construct a new input sample.
    #[must_use]
    pub const fn new(x: f32, y: f32, pressure: f32) -> Self {
        Self { x, y, pressure }
    }
}

/// Configuration for [`crate::get_stroke`].
///
/// All ratios are in `0.0..=1.0` unless noted otherwise. The defaults
/// match the perfect-freehand TypeScript defaults closely enough that
/// strokes look visually equivalent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeOptions {
    /// Base stroke width (diameter) in world units. Must be positive
    /// for any output to be produced.
    pub size: f32,
    /// Effect of pressure on width, in `-1.0..=1.0`.
    ///
    /// `0.0` disables pressure thinning. Positive values make the
    /// stroke thinner under low pressure; negative values invert.
    pub thinning: f32,
    /// How much to soften the stroke's edges, in `0.0..=1.0`.
    ///
    /// Larger values drop more redundant outline points so the
    /// resulting polygon has fewer vertices on long straight runs.
    pub smoothing: f32,
    /// Input streamlining factor, in `0.0..=1.0`.
    ///
    /// Higher values smooth jittery input by pulling each new sample
    /// toward the previous adjusted point.
    pub streamline: f32,
    /// Length of the tapered start, expressed as a fraction of
    /// `max(size, total_length)`. `0.0` disables the start taper;
    /// `1.0` tapers over the whole stroke.
    pub start_taper: f32,
    /// Length of the tapered end, expressed as a fraction of
    /// `max(size, total_length)`. Same convention as `start_taper`.
    pub end_taper: f32,
    /// When `true`, pressure is derived from drawing velocity. Use
    /// this for devices that do not report pressure (mouse, touch).
    pub simulate_pressure: bool,
}

impl Default for StrokeOptions {
    fn default() -> Self {
        Self {
            size: 16.0,
            thinning: 0.5,
            smoothing: 0.5,
            streamline: 0.5,
            start_taper: 0.0,
            end_taper: 0.0,
            simulate_pressure: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_match_original() {
        let o = StrokeOptions::default();
        assert!((o.size - 16.0).abs() < f32::EPSILON);
        assert!((o.thinning - 0.5).abs() < f32::EPSILON);
        assert!((o.smoothing - 0.5).abs() < f32::EPSILON);
        assert!((o.streamline - 0.5).abs() < f32::EPSILON);
        assert!(o.simulate_pressure);
    }

    #[test]
    fn input_point_constructor() {
        let p = InputPoint::new(1.0, 2.0, 0.75);
        assert!((p.x - 1.0).abs() < f32::EPSILON);
        assert!((p.y - 2.0).abs() < f32::EPSILON);
        assert!((p.pressure - 0.75).abs() < f32::EPSILON);
    }
}
