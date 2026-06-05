//! `GET /api/info` — server identity and protocol version.

use axum::{Json, response::IntoResponse};
use bse_protocol::PROTOCOL_VERSION;
use serde::Serialize;

/// Server identification advertised to clients.
///
/// Returned by `GET /api/info`. Clients should use [`Self::protocol`]
/// to decide whether they can speak to this server.
#[derive(Debug, Serialize)]
pub struct ServerInfo {
    /// Human-readable server name.
    pub name: &'static str,
    /// Server binary version (from `CARGO_PKG_VERSION`).
    pub version: &'static str,
    /// Wire protocol version, mirrors [`bse_protocol::PROTOCOL_VERSION`].
    pub protocol: &'static str,
}

/// Static metadata baked into the binary at compile time.
pub const SERVER_INFO: ServerInfo = ServerInfo {
    name: "BSE Server",
    version: env!("CARGO_PKG_VERSION"),
    protocol: PROTOCOL_VERSION,
};

/// Respond with a JSON dump of [`SERVER_INFO`].
pub async fn info() -> impl IntoResponse {
    Json(&SERVER_INFO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn returns_protocol_and_version() {
        let resp = info().await.into_response();
        assert_eq!(resp.status(), 200);
        let body = to_bytes(resp.into_body(), 1024).await.expect("read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(json["protocol"], PROTOCOL_VERSION);
        assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(json["name"], "BSE Server");
    }
}
