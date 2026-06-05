//! BSE collaboration server library.
//!
//! The `bse-server` binary is a thin wrapper around this library so
//! that integration tests in `tests/` can exercise the same router
//! via `tower::ServiceExt::oneshot`.
//!
//! See the binary entry point in `src/main.rs` for the runtime
//! glue (signal handling, listener binding, tracing init).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod config;
pub mod handlers;
pub mod rooms;
pub mod routes;
pub mod state;
pub mod users;
pub mod ws;

pub use config::ServerConfig;
pub use routes::router;
pub use state::AppState;
