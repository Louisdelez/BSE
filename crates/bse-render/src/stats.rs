//! Per-frame statistics, useful for the in-app overlay and profiling.

/// Statistics gathered during a single frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameStats {
    /// Total number of elements considered (before culling).
    pub elements_total: u32,
    /// Number of elements after viewport culling.
    pub elements_visible: u32,
    /// Number of draw calls emitted.
    pub draw_calls: u32,
    /// CPU time spent preparing the frame, in microseconds.
    pub cpu_prepare_us: u32,
}
