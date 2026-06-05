//! Centralised tracing/logging configuration.

use tracing_subscriber::EnvFilter;

/// Initialize the global tracing subscriber.
///
/// Reads the `RUST_LOG` env var (e.g. `RUST_LOG=bse=debug`) and falls
/// back to `info` if unset.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
