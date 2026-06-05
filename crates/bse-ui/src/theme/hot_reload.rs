//! Hot-reload of design tokens — watch a RON file and call a closure
//! whenever it changes.
//!
//! Inspired by Rerun's `re_ui::hot_reload_design_tokens` pattern :
//! cfg-gated, dev-only, opt-in via the `hot-reload-tokens` feature.
//! When the feature is off, [`start_hot_reload`] is a no-op stub.

#[cfg(feature = "hot-reload-tokens")]
mod active {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;
    use std::time::Duration;

    use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use serde::Deserialize;
    use tracing::{info, warn};

    /// Tokens deserialized from RON.
    #[derive(Debug, Deserialize)]
    #[allow(missing_docs)] // fields are 1:1 with the RON keys ; doc on the struct
    pub struct ThemeTokens {
        pub name: String,
        pub colors: ColorTokens,
        pub radii: RadiusTokens,
        pub spacing: SpacingTokens,
    }

    /// Colors block of the RON token file (hex strings).
    #[derive(Debug, Deserialize)]
    #[allow(missing_docs)]
    pub struct ColorTokens {
        pub brand_yellow: String,
        pub brand_blue: String,
        pub ink: String,
        pub canvas: String,
        pub surface: String,
        pub hairline: String,
        pub success: String,
        pub warning: String,
        pub error: String,
    }

    /// Corner-radius scale.
    #[derive(Debug, Deserialize)]
    #[allow(missing_docs)]
    pub struct RadiusTokens {
        pub sm: f32,
        pub md: f32,
        pub lg: f32,
        pub xxxl: f32,
        pub pill: f32,
    }

    /// Spacing scale (4-px grid).
    #[derive(Debug, Deserialize)]
    #[allow(missing_docs)]
    pub struct SpacingTokens {
        pub xs: f32,
        pub sm: f32,
        pub md: f32,
        pub lg: f32,
        pub xl: f32,
    }

    /// Spawn a background thread watching `path`. On every successful
    /// reload, `on_change(tokens)` is called from that thread.
    ///
    /// Returns the spawned watcher handle ; drop it to stop watching.
    #[allow(clippy::needless_pass_by_value)] // PathBuf moved into watcher / thread by clones
    pub fn start_hot_reload<F>(path: PathBuf, mut on_change: F) -> Option<RecommendedWatcher>
    where
        F: FnMut(ThemeTokens) + Send + 'static,
    {
        if !path.exists() {
            warn!(
                target: "bse::ui::hot_reload",
                path = %path.display(),
                "token file does not exist ; hot-reload disabled",
            );
            return None;
        }

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let watcher_result = RecommendedWatcher::new(
            tx,
            Config::default().with_poll_interval(Duration::from_millis(400)),
        );
        let mut watcher = match watcher_result {
            Ok(w) => w,
            Err(err) => {
                warn!(target: "bse::ui::hot_reload", error = %err, "failed to create watcher");
                return None;
            }
        };

        if let Err(err) = watcher.watch(&path, RecursiveMode::NonRecursive) {
            warn!(
                target: "bse::ui::hot_reload",
                path = %path.display(),
                error = %err,
                "failed to watch token file",
            );
            return None;
        }

        // Initial load.
        if let Some(tokens) = read_tokens(&path) {
            info!(target: "bse::ui::hot_reload", "initial tokens loaded from {}", path.display());
            on_change(tokens);
        }

        let path_clone = path.clone();
        std::thread::Builder::new()
            .name("bse-ui-hot-reload".into())
            .spawn(move || {
                for ev in rx {
                    let Ok(ev) = ev else { continue };
                    if matches!(ev.kind, EventKind::Modify(_) | EventKind::Create(_))
                        && let Some(tokens) = read_tokens(&path_clone)
                    {
                        info!(target: "bse::ui::hot_reload", "tokens reloaded");
                        on_change(tokens);
                    }
                }
            })
            .ok();

        Some(watcher)
    }

    fn read_tokens(path: &Path) -> Option<ThemeTokens> {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) => {
                warn!(target: "bse::ui::hot_reload", error = %err, "read token file failed");
                return None;
            }
        };
        match ron::from_str::<ThemeTokens>(&raw) {
            Ok(t) => Some(t),
            Err(err) => {
                warn!(target: "bse::ui::hot_reload", error = %err, "RON parse error ; keeping old tokens");
                None
            }
        }
    }
}

#[cfg(feature = "hot-reload-tokens")]
pub use active::{ColorTokens, RadiusTokens, SpacingTokens, ThemeTokens, start_hot_reload};

#[cfg(not(feature = "hot-reload-tokens"))]
mod stub {
    use std::path::PathBuf;

    /// No-op stub when the `hot-reload-tokens` feature is off.
    /// Always returns `None`.
    pub fn start_hot_reload<F>(_path: PathBuf, _on_change: F) -> Option<()>
    where
        F: FnMut(()) + Send + 'static,
    {
        None
    }
}

#[cfg(not(feature = "hot-reload-tokens"))]
pub use stub::start_hot_reload;
