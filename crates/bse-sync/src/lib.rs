//! WebSocket client used by the BSE app to talk to a BSE server.
//!
//! v002 only defines the connection state enum and a stub. The real
//! tokio-tungstenite client lands in v008 (server foundation milestone).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use thiserror::Error;

/// Errors produced by the sync client.
#[derive(Debug, Error)]
pub enum SyncError {
    /// The server is unreachable.
    #[error("connection failed : {0}")]
    Connection(String),

    /// Feature not yet implemented at this milestone.
    #[error("not yet implemented : {0}")]
    NotImplemented(&'static str),
}

/// Coarse-grained connection state, displayed to the user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConnectionState {
    /// No server configured.
    #[default]
    Offline,
    /// Currently attempting to connect.
    Connecting,
    /// Connected and synchronizing.
    Connected,
    /// Lost connection, retrying with exponential backoff.
    Reconnecting,
}

impl ConnectionState {
    /// Short human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Offline => "Offline",
            Self::Connecting => "Connecting...",
            Self::Connected => "Connected",
            Self::Reconnecting => "Reconnecting...",
        }
    }
}
