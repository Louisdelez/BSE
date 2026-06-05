//! Static app metadata used by various UI surfaces.

/// Static metadata about the running app.
#[derive(Clone, Copy, Debug)]
pub struct AppInfo {
    /// Marketing name, e.g. `"BSE"`.
    pub name: &'static str,
    /// Semver of the binary, e.g. `env!("CARGO_PKG_VERSION")`.
    pub version: &'static str,
    /// Active milestone tag, e.g. `"v002"`.
    pub milestone: &'static str,
}

impl AppInfo {
    /// Compose a `"BSE 0.0.2 — v002"` style title bar string.
    #[must_use]
    pub fn title(&self) -> String {
        format!("{} {} — {}", self.name, self.version, self.milestone)
    }
}
