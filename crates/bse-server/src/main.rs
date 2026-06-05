//! BSE collaboration server entry point.
//!
//! In v002 the binary prints its version and protocol info, then exits.
//! The real axum + tokio server lands in v008.

use bse_protocol::PROTOCOL_VERSION;
use tracing::info;

const SERVER_NAME: &str = "BSE Server";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    init_tracing();
    info!("starting {SERVER_NAME} {VERSION}");
    info!("speaking protocol {PROTOCOL_VERSION}");

    println!("✓ {SERVER_NAME} {VERSION} ready (protocol {PROTOCOL_VERSION})");
    println!("   ↳ HTTP / WebSocket endpoints arrive in v008.");
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
