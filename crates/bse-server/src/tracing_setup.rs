//! Centralised tracing/logging configuration for the server binary.
//!
//! Mirrors `bse-app`'s `tracing_setup` so log formats match across the
//! two binaries. The server uses `RUST_LOG` to gate verbosity and
//! falls back to `info` when unset.

use tracing_subscriber::EnvFilter;

/// Initialize the global tracing subscriber.
///
/// Reads the `RUST_LOG` env var (e.g. `RUST_LOG=bse_server=debug,tower_http=debug`)
/// and falls back to `info` if unset.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
