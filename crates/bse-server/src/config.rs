//! Server configuration loaded from environment variables.
//!
//! For v008 the only configurable knob is the bind address. Future
//! milestones will add TLS, max-connection limits, and persistence
//! paths here.

use std::net::SocketAddr;

/// Environment variable controlling the listen address.
pub const BIND_ADDR_ENV: &str = "BSE_BIND_ADDR";

/// Default bind address when no env var is set.
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

/// Runtime configuration for the collaboration server.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Socket address the HTTP listener will bind to.
    pub bind_addr: SocketAddr,
}

impl ServerConfig {
    /// Load configuration from the process environment.
    ///
    /// Reads `BSE_BIND_ADDR` and falls back to [`DEFAULT_BIND_ADDR`]
    /// when unset. If the variable contains a malformed address the
    /// default is used and a warning is logged.
    #[must_use]
    pub fn from_env() -> Self {
        let raw = std::env::var(BIND_ADDR_ENV).unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned());
        let bind_addr = raw.parse::<SocketAddr>().unwrap_or_else(|err| {
            tracing::warn!(
                value = %raw,
                error = %err,
                "invalid {BIND_ADDR_ENV}, falling back to {DEFAULT_BIND_ADDR}"
            );
            DEFAULT_BIND_ADDR
                .parse()
                .expect("hard-coded default must parse")
        });
        Self { bind_addr }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: DEFAULT_BIND_ADDR
                .parse()
                .expect("hard-coded default must parse"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_uses_canonical_port() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.bind_addr.port(), 8080);
    }

    #[test]
    fn default_bind_addr_parses() {
        let parsed: SocketAddr = DEFAULT_BIND_ADDR.parse().expect("must parse");
        assert_eq!(parsed.port(), 8080);
    }
}
