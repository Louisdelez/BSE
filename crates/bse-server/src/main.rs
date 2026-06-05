//! BSE collaboration server entry point.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod tracing_setup;

use bse_protocol::PROTOCOL_VERSION;
use bse_server::{AppState, ServerConfig, routes::router_with};
use tracing::{error, info};

const SERVER_NAME: &str = "BSE Server";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    tracing_setup::init();
    info!("starting {SERVER_NAME} {VERSION}");
    info!("speaking protocol {PROTOCOL_VERSION}");

    let cfg = ServerConfig::from_env();
    info!(addr = %cfg.bind_addr, data_dir = %cfg.data_dir.display(), "configuration loaded");

    if let Err(err) = serve(cfg).await {
        error!(error = %err, "server exited with error");
        std::process::exit(1);
    }
}

/// Bind the TCP listener and run the axum service until shutdown.
async fn serve(cfg: ServerConfig) -> std::io::Result<()> {
    let state = AppState::from_config(&cfg);
    let app = router_with(state);
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
