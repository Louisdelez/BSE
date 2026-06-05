//! Motion tokens — durations and easings used across BSE's animations.
//!
//! Inspired by Material 3's expressive motion tokens and Linear /
//! Vercel's product motion. Tokenizing durations + easings the same
//! way colors are tokenized keeps the app feeling consistent across
//! screens.

use std::time::Duration;

// ─── Durations ───────────────────────────────────────────────────────────────

/// Hover / focus transitions. Below 100ms the change is invisible ;
/// above 150ms it starts to feel sluggish.
pub const DURATION_MICRO: Duration = Duration::from_millis(120);

/// Component reveal — modal scale, panel slide, toast slide-in.
pub const DURATION_STANDARD: Duration = Duration::from_millis(200);

/// Macro transitions — page-level changes, room switches.
pub const DURATION_MACRO: Duration = Duration::from_millis(350);

// ─── Easings ─────────────────────────────────────────────────────────────────

/// "Vercel curve" — cubic-bezier(0.16, 1.0, 0.3, 1.0). System-driven
/// motion (modals, menus, panels). Easy entrance, calm settle.
pub const EASE_STANDARD: [f32; 4] = [0.16, 1.0, 0.3, 1.0];

/// "Back-out" — cubic-bezier(0.34, 1.56, 0.64, 1.0). User-driven
/// motion (drops, stamps). Slight overshoot for tactile feel.
pub const EASE_BACK_OUT: [f32; 4] = [0.34, 1.56, 0.64, 1.0];

/// Linear — pretty rare in good motion design ; reserved for
/// progress bars and looping spinners.
pub const EASE_LINEAR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

// ─── Spring presets (for egui_animation crate use) ───────────────────────────

/// Default spring stiffness — soft and natural feel.
pub const SPRING_DEFAULT_STIFFNESS: f32 = 300.0;
/// Default spring damping — pairs with [`SPRING_DEFAULT_STIFFNESS`].
pub const SPRING_DEFAULT_DAMPING: f32 = 20.0;

/// Fast spring stiffness — snappy state toggles.
pub const SPRING_FAST_STIFFNESS: f32 = 500.0;
/// Fast spring damping — pairs with [`SPRING_FAST_STIFFNESS`].
pub const SPRING_FAST_DAMPING: f32 = 30.0;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Evaluate a cubic-bezier curve at parametric `t` in `[0, 1]`. Used
/// when we need an easing curve outside of `Context::animate_*`.
///
/// `c = [x1, y1, x2, y2]` are the two Bézier control points.
#[must_use]
pub fn cubic_bezier(c: [f32; 4], t: f32) -> f32 {
    // Newton-Raphson approximation, plenty good for UI use (< 1e-3 error).
    let x1 = c[0];
    let y1 = c[1];
    let x2 = c[2];
    let y2 = c[3];

    // Solve cubic-bezier x(p) = t for p, then return y(p).
    let mut p = t;
    for _ in 0..8 {
        let x = bezier(x1, x2, p);
        let dx = bezier_d(x1, x2, p);
        if dx.abs() < 1e-6 {
            break;
        }
        p -= (x - t) / dx;
    }
    p = p.clamp(0.0, 1.0);
    bezier(y1, y2, p)
}

fn bezier(a: f32, b: f32, t: f32) -> f32 {
    // Bezier coefficients for x(t) = 3(1-t)^2 t * a + 3(1-t) t^2 * b + t^3
    3.0 * (1.0 - t).powi(2) * t * a + 3.0 * (1.0 - t) * t.powi(2) * b + t.powi(3)
}

fn bezier_d(a: f32, b: f32, t: f32) -> f32 {
    3.0 * (1.0 - t).powi(2) * a + 6.0 * (1.0 - t) * t * (b - a) + 3.0 * t.powi(2) * (1.0 - b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_bezier_endpoints() {
        let c = EASE_STANDARD;
        assert!((cubic_bezier(c, 0.0)).abs() < 1e-3);
        assert!((cubic_bezier(c, 1.0) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn cubic_bezier_back_out_overshoots() {
        // Back-out curves rise above 1.0 mid-way before settling.
        let c = EASE_BACK_OUT;
        let peak = cubic_bezier(c, 0.75);
        assert!(peak >= 0.95, "got {peak}");
    }
}
