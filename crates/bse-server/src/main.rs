//! BSE collaboration server entry point.
//!
//! Starts an axum HTTP listener with the v008 routes wired up (see
//! [`bse_server::router`]). Shuts down cleanly on Ctrl-C.
//!
//! Configuration is read from the environment via
//! [`bse_server::ServerConfig::from_env`]; see that module for the list
//! of supported variables.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod tracing_setup;

use bse_protocol::PROTOCOL_VERSION;
use bse_server::{ServerConfig, router};
use tracing::{error, info};

const SERVER_NAME: &str = "BSE Server";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    tracing_setup::init();
    info!("starting {SERVER_NAME} {VERSION}");
    info!("speaking protocol {PROTOCOL_VERSION}");

    let cfg = ServerConfig::from_env();
    if let Err(err) = serve(cfg).await {
        error!(error = %err, "server exited with error");
        std::process::exit(1);
    }
}

/// Bind the TCP listener and run the axum service until shutdown.
async fn serve(cfg: ServerConfig) -> std::io::Result<()> {
    let app = router();
    let listener = tokio::net::TcpListener::bind(cfg.bind_addr).await?;
    let local = listener.local_addr()?;
    info!(addr = %local, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("server shut down cleanly");
    Ok(())
}

/// Resolve when the user sends SIGINT / Ctrl-C.
async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => info!("ctrl-c received, shutting down"),
        Err(err) => error!(error = %err, "failed to install ctrl-c handler"),
    }
}
