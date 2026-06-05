//! Error type returned by every fallible operation in this crate.

use thiserror::Error;

/// Errors produced by the sync client.
///
/// All variants are `String`-based to avoid leaking the concrete
/// `tokio-tungstenite` / `rmp_serde` error types into the public API.
#[derive(Debug, Error)]
pub enum SyncError {
    /// Transport-level failure : the WebSocket could not be opened,
    /// or the peer closed the connection unexpectedly.
    #[error("connection failed : {0}")]
    Connection(String),

    /// Wire-format error : `MessagePack` encode/decode failed, or a frame
    /// arrived with an unsupported type (e.g. raw `Ping`/`Pong` outside
    /// the BSE protocol envelope).
    #[error("protocol error : {0}")]
    Protocol(String),

    /// Feature not yet implemented at this milestone.
    ///
    /// Reserved for v0XX milestones that have not landed yet. Kept here
    /// to preserve the public API documented in the v002 stub.
    #[error("not yet implemented : {0}")]
    NotImplemented(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_format_matches_spec() {
        let e = SyncError::Connection("refused".into());
        assert_eq!(e.to_string(), "connection failed : refused");
        let e = SyncError::Protocol("bad msgpack".into());
        assert_eq!(e.to_string(), "protocol error : bad msgpack");
        let e = SyncError::NotImplemented("reconnect");
        assert_eq!(e.to_string(), "not yet implemented : reconnect");
    }
}
