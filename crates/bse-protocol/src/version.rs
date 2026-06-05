//! Protocol version constant.

/// Version string advertised in the WebSocket subprotocol header.
///
/// Bump this whenever an incompatible change is made to the message
/// schema. The server is expected to support `current` and `current - 1`.
pub const PROTOCOL_VERSION: &str = "bse.v1";
