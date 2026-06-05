//! Second pass: build the closed outline polygon from
//! [`StrokePoint`]s.
//!
//! Direct port of `getStrokeOutlinePoints.ts` from perfect-freehand,
//! using only the features the public BSE API exposes (no custom
//! easing functions, no per-end cap toggles — taper > 0 disables the
//! cap on that end automatically).

use bse_types::Vec2;

use crate::options::StrokeOptions;
use crate::stroke_points::StrokePoint;
use crate::vec::{FIXED_PI, dist2, dpr, neg, per, prj, rot_around, uni};

const START_CAP_SEGMENTS: u32 = 13;
const END_CAP_SEGMENTS: u32 = 29;
const CORNER_CAP_SEGMENTS: u32 = 13;
const END_NOISE_THRESHOLD: f32 = 3.0;
const MIN_RADIUS: f32 = 0.01;
const RATE_OF_PRESSURE_CHANGE: f32 = 0.275;

/// Compute the half-width of the stroke at a given pressure.
fn stroke_radius(size: f32, thinning: f32, pressure: f32) -> f32 {
    // Easing is identity in v006. TODO(v006.1): configurable easing.
    size * (0.5 - thinning * (0.5 - pressure))
}

/// Heuristic that derives a pressure value from drawing speed.
fn simulate_pressure(prev: f32, distance: f32, size: f32) -> f32 {
    let sp = (distance / size).min(1.0);
    let rp = (1.0_f32 - sp).min(1.0);
    (prev + (rp - prev) * (sp * RATE_OF_PRESSURE_CHANGE)).min(1.0)
}

/// Convert the v006 taper ratio into an absolute distance.
fn taper_distance(taper: f32, size: f32, total_length: f32) -> f32 {
    let taper = taper.clamp(0.0, 1.0);
    if taper <= 0.0 {
        0.0
    } else {
        size.max(total_length) * taper
    }
}

/// Easing used at the start taper. Matches the TS default `t * (2 - t)`.
fn ease_start(t: f32) -> f32 {
    t * (2.0 - t)
}

/// Easing used at the end taper. Matches the TS default
/// `(t - 1)^3 + 1`.
fn ease_end(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

/// Average pressure over the first few samples to avoid fat starts.
fn initial_pressure(points: &[StrokePoint], simulate: bool, size: f32) -> f32 {
    let take = points.len().min(10);
    points[..take].iter().fold(points[0].pressure, |acc, curr| {
        let p = if simulate {
            simulate_pressure(acc, curr.distance, size)
        } else {
            curr.pressure
        };
        f32::midpoint(acc, p)
    })
}

/// Outline points for a degenerate (single-sample) stroke.
fn draw_dot(center: Vec2, radius: f32) -> Vec<Vec2> {
    let offset = Vec2::new(1.0, 1.0);
    let start = prj(center, uni(per(center - (center + offset))), -radius);
    let mut out = Vec::with_capacity(START_CAP_SEGMENTS as usize);
    let step = 1.0 / f32::from(u16::try_from(START_CAP_SEGMENTS).unwrap_or(u16::MAX));
    let mut t = step;
    while t <= 1.0 + f32::EPSILON {
        out.push(rot_around(start, center, FIXED_PI * 2.0 * t));
        t += step;
    }
    out
}

/// Rounded start cap: half-turn arc from the right side over to the
/// left side of the first stroke point.
fn round_start_cap(center: Vec2, right_point: Vec2, segments: u32) -> Vec<Vec2> {
    let mut cap = Vec::with_capacity(segments as usize);
    let step = 1.0 / f32::from(u16::try_from(segments).unwrap_or(u16::MAX));
    let mut t = step;
    while t <= 1.0 + f32::EPSILON {
        cap.push(rot_around(right_point, center, FIXED_PI * t));
        t += step;
    }
    cap
}

/// Rounded end cap: 1.5-turn arc to absorb any sharp final turn.
fn round_end_cap(center: Vec2, direction: Vec2, radius: f32, segments: u32) -> Vec<Vec2> {
    let mut cap = Vec::with_capacity(segments as usize);
    let start = prj(center, direction, radius);
    let step = 1.0 / f32::from(u16::try_from(segments).unwrap_or(u16::MAX));
    let mut t = step;
    while t < 1.0 {
        cap.push(rot_around(start, center, FIXED_PI * 3.0 * t));
        t += step;
    }
    cap
}

/// Compute the closed outline polygon. See [`crate::get_stroke`] for
/// the public entry point.
#[allow(clippy::too_many_lines)] // hot loop with several inline cases
pub(crate) fn get_stroke_outline_points(
    points: &[StrokePoint],
    options: &StrokeOptions,
) -> Vec<Vec2> {
    if points.is_empty() || options.size <= 0.0 {
        return Vec::new();
    }

    let size = options.size;
    let smoothing = options.smoothing.clamp(0.0, 1.0);
    let thinning = options.thinning.clamp(-1.0, 1.0);

    let total_length = points[points.len() - 1].running_length;
    let taper_start = taper_distance(options.start_taper, size, total_length);
    let taper_end = taper_distance(options.end_taper, size, total_length);
    let min_distance = (size * smoothing).powi(2);

    let mut left_pts: Vec<Vec2> = Vec::with_capacity(points.len());
    let mut right_pts: Vec<Vec2> = Vec::with_capacity(points.len());

    let mut prev_pressure = initial_pressure(points, options.simulate_pressure, size);
    let mut radius = stroke_radius(size, thinning, points[points.len() - 1].pressure);
    let mut first_radius: Option<f32> = None;
    let mut prev_vector = points[0].vector;
    let mut prev_left = points[0].point;
    let mut prev_right = points[0].point;
    let mut temp_left = prev_left;
    let mut temp_right = prev_right;
    let mut is_prev_sharp = false;

    for i in 0..points.len() {
        let sp = points[i];
        let mut pressure = sp.pressure;
        let is_last = i == points.len() - 1;

        // Skip end noise — but always honor the last point itself.
        if !is_last && total_length - sp.running_length < END_NOISE_THRESHOLD {
            continue;
        }

        if thinning.abs() > 0.0 {
            if options.simulate_pressure {
                pressure = simulate_pressure(prev_pressure, sp.distance, size);
            }
            radius = stroke_radius(size, thinning, pressure);
        } else {
            radius = size / 2.0;
        }
        if first_radius.is_none() {
            first_radius = Some(radius);
        }

        let taper_start_strength = if sp.running_length < taper_start && taper_start > 0.0 {
            ease_start(sp.running_length / taper_start)
        } else {
            1.0
        };
        let taper_end_strength = if total_length - sp.running_length < taper_end && taper_end > 0.0
        {
            ease_end((total_length - sp.running_length) / taper_end)
        } else {
            1.0
        };
        radius = MIN_RADIUS.max(radius * taper_start_strength.min(taper_end_strength));

        let next_vector = if is_last {
            sp.vector
        } else {
            points[i + 1].vector
        };
        let next_dpr = if is_last {
            1.0
        } else {
            dpr(sp.vector, next_vector)
        };
        let prev_dpr = dpr(sp.vector, prev_vector);

        let is_point_sharp = prev_dpr < 0.0 && !is_prev_sharp;
        let is_next_sharp = next_dpr < 0.0;

        if is_point_sharp || is_next_sharp {
            // Walk a half-turn cap around the current point.
            let offset = per(prev_vector) * radius;
            let step = 1.0 / f32::from(u16::try_from(CORNER_CAP_SEGMENTS).unwrap_or(u16::MAX));
            let mut t = 0.0_f32;
            while t <= 1.0 + f32::EPSILON {
                let tl = rot_around(sp.point - offset, sp.point, FIXED_PI * t);
                let tr = rot_around(sp.point + offset, sp.point, FIXED_PI * -t);
                left_pts.push(tl);
                right_pts.push(tr);
                temp_left = tl;
                temp_right = tr;
                t += step;
            }

            prev_left = temp_left;
            prev_right = temp_right;
            if is_next_sharp {
                is_prev_sharp = true;
            }
            continue;
        }

        is_prev_sharp = false;

        if is_last {
            let offset = per(sp.vector) * radius;
            left_pts.push(sp.point - offset);
            right_pts.push(sp.point + offset);
            continue;
        }

        // Regular interior point: shift perpendicular to the average
        // of the incoming and outgoing tangents.
        let lerped = next_vector + (sp.vector - next_vector) * next_dpr;
        let offset = per(lerped) * radius;

        let tl = sp.point - offset;
        if i <= 1 || dist2(prev_left, tl) > min_distance {
            left_pts.push(tl);
            prev_left = tl;
        }
        let tr = sp.point + offset;
        if i <= 1 || dist2(prev_right, tr) > min_distance {
            right_pts.push(tr);
            prev_right = tr;
        }

        prev_pressure = pressure;
        prev_vector = sp.vector;
    }

    let first_point = points[0].point;
    let last_point = if points.len() > 1 {
        points[points.len() - 1].point
    } else {
        points[0].point + Vec2::new(1.0, 1.0)
    };

    let mut start_cap: Vec<Vec2> = Vec::new();
    let mut end_cap: Vec<Vec2> = Vec::new();

    // Single-point stroke: render as a dot when no taper is requested.
    if points.len() == 1 {
        if taper_start <= 0.0 && taper_end <= 0.0 {
            return draw_dot(first_point, first_radius.unwrap_or(radius));
        }
    } else {
        if taper_start > 0.0 || (taper_end > 0.0 && points.len() == 1) {
            // Tapered start — nothing to do.
        } else if let Some(&first_right) = right_pts.first() {
            start_cap.extend(round_start_cap(
                first_point,
                first_right,
                START_CAP_SEGMENTS,
            ));
        }

        let direction = per(neg(points[points.len() - 1].vector));
        if taper_end > 0.0 || (taper_start > 0.0 && points.len() == 1) {
            end_cap.push(last_point);
        } else {
            end_cap.extend(round_end_cap(
                last_point,
                direction,
                radius,
                END_CAP_SEGMENTS,
            ));
        }
    }

    // Stitch left side -> end cap -> reversed right side -> start cap.
    let mut out = left_pts;
    out.extend(end_cap);
    out.extend(right_pts.into_iter().rev());
    out.extend(start_cap);
    out
}
