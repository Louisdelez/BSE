//! First pass: turn raw input samples into smoothed
//! [`StrokePoint`]s with running length and tangent vectors.
//!
//! Direct port of `getStrokePoints.ts` from perfect-freehand.

use bse_types::Vec2;

use crate::options::{InputPoint, StrokeOptions};
use crate::vec::{approx_eq, uni};

/// Default pressure for the first point. Lower than the rest to
/// avoid a fat start since real strokes always begin slow.
pub(crate) const DEFAULT_FIRST_PRESSURE: f32 = 0.25;

/// Default pressure for subsequent points when none is reported.
pub(crate) const DEFAULT_PRESSURE: f32 = 0.5;

/// Minimum streamline interpolation factor (used when `streamline == 1`).
const MIN_STREAMLINE_T: f32 = 0.15;

/// Range added to [`MIN_STREAMLINE_T`] based on `1 - streamline`.
const STREAMLINE_T_RANGE: f32 = 0.85;

/// Placeholder unit offset used when a stroke has only one input point.
const UNIT_OFFSET: Vec2 = Vec2::new(1.0, 1.0);

/// One sample after streamlining, with cached tangent and arc length.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StrokePoint {
    /// Smoothed position.
    pub point: Vec2,
    /// Pressure attached to this sample, in `0.0..=1.0`.
    pub pressure: f32,
    /// Unit tangent **from this point toward the previous point**
    /// (matches the TS `vector` field).
    pub vector: Vec2,
    /// Distance from the previous smoothed point.
    pub distance: f32,
    /// Total arc length up to and including this point.
    pub running_length: f32,
}

/// Convert raw input into smoothed [`StrokePoint`]s.
///
/// Returns an empty `Vec` when the input is empty. When there is only
/// one input point, a second one is synthesised at a unit offset so
/// that downstream code can still produce a dot. When there are
/// exactly two, three intermediate points are inserted so taper logic
/// has something to walk along.
pub(crate) fn get_stroke_points(
    points: &[InputPoint],
    options: &StrokeOptions,
) -> Vec<StrokePoint> {
    if points.is_empty() {
        return Vec::new();
    }

    let t = MIN_STREAMLINE_T + (1.0 - options.streamline.clamp(0.0, 1.0)) * STREAMLINE_T_RANGE;

    // Materialise the input as a working `Vec` so we can pad it
    // without mutating the caller's slice.
    let mut pts: Vec<InputPoint> = points.to_vec();

    // For a two-point stroke, expand into five evenly spaced samples
    // so tapered start/end caps have room to fade.
    if pts.len() == 2 {
        let first = pts[0];
        let last = pts[1];
        pts.clear();
        pts.push(first);
        for i in 1..5_u32 {
            let f = f32::from(u16::try_from(i).unwrap_or(0)) / 4.0;
            pts.push(InputPoint::new(
                first.x + (last.x - first.x) * f,
                first.y + (last.y - first.y) * f,
                // Linearly interpolate pressure to match TS lrp.
                first.pressure + (last.pressure - first.pressure) * f,
            ));
        }
    }

    // For a one-point stroke, synthesise a second point a unit away.
    if pts.len() == 1 {
        let p = pts[0];
        pts.push(InputPoint::new(
            p.x + UNIT_OFFSET.x,
            p.y + UNIT_OFFSET.y,
            p.pressure,
        ));
    }

    let mut stroke_points: Vec<StrokePoint> = Vec::with_capacity(pts.len());
    stroke_points.push(StrokePoint {
        point: Vec2::new(pts[0].x, pts[0].y),
        pressure: if pts[0].pressure >= 0.0 {
            pts[0].pressure
        } else {
            DEFAULT_FIRST_PRESSURE
        },
        vector: UNIT_OFFSET,
        distance: 0.0,
        running_length: 0.0,
    });

    let mut has_reached_minimum_length = false;
    let mut running_length = 0.0_f32;
    let mut prev_point = stroke_points[0].point;
    let max = pts.len() - 1;

    for (i, raw_input) in pts.iter().enumerate().skip(1) {
        let raw = Vec2::new(raw_input.x, raw_input.y);
        let point = prev_point + (raw - prev_point) * t;

        if approx_eq(prev_point, point) {
            continue;
        }

        let distance = point.distance(prev_point);
        running_length += distance;

        // Skip the noisy beginning until we are at least `size`
        // pixels from the start, but never drop the last point.
        if i < max && !has_reached_minimum_length {
            if running_length < options.size {
                prev_point = point;
                continue;
            }
            has_reached_minimum_length = true;
            // TODO(v006.1): backfill skipped points so start tapers
            // are continuous with the rest of the stroke. The TS
            // original carries the same TODO.
        }

        let vector = uni(prev_point - point);
        let pressure = if raw_input.pressure >= 0.0 {
            raw_input.pressure
        } else {
            DEFAULT_PRESSURE
        };

        stroke_points.push(StrokePoint {
            point,
            pressure,
            vector,
            distance,
            running_length,
        });

        prev_point = point;
    }

    // Copy the second point's vector back onto the first so the start
    // cap orientation matches the stroke direction.
    if stroke_points.len() >= 2 {
        stroke_points[0].vector = stroke_points[1].vector;
    } else {
        stroke_points[0].vector = Vec2::ZERO;
    }

    stroke_points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_empty_output() {
        let out = get_stroke_points(&[], &StrokeOptions::default());
        assert!(out.is_empty());
    }

    #[test]
    fn single_point_is_padded_to_two() {
        let out = get_stroke_points(&[InputPoint::new(0.0, 0.0, 0.5)], &StrokeOptions::default());
        assert!(!out.is_empty());
        assert!((out[0].point.x).abs() < 1e-6);
    }

    #[test]
    fn two_points_expand_into_more_samples() {
        let pts = [
            InputPoint::new(0.0, 0.0, 0.5),
            InputPoint::new(100.0, 0.0, 0.5),
        ];
        let opts = StrokeOptions {
            streamline: 0.0,
            size: 1.0,
            ..Default::default()
        };
        let out = get_stroke_points(&pts, &opts);
        assert!(
            out.len() >= 3,
            "expected the 2-point expansion, got {}",
            out.len()
        );
    }
}
