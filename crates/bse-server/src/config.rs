//! Server configuration loaded from environment variables.

use std::net::SocketAddr;
use std::path::PathBuf;

/// Environment variable controlling the listen address.
pub const BIND_ADDR_ENV: &str = "BSE_BIND_ADDR";

/// Default bind address when no env var is set.
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

/// Environment variable controlling the on-disk data directory used by
/// [`crate::store::ServerStore`] (users, room snapshots).
pub const DATA_DIR_ENV: &str = "BSE_SERVER_DATA_DIR";

/// Default data directory when `BSE_SERVER_DATA_DIR` is not set.
pub const DEFAULT_DATA_DIR: &str = "data";

const SQLITE_FILENAME: &str = "server.sqlite";

/// Runtime configuration for the collaboration server.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Socket address the HTTP listener will bind to.
    pub bind_addr: SocketAddr,
    /// Directory used to store the `SQLite` database (`server.sqlite`).
    pub data_dir: PathBuf,
}

impl ServerConfig {
    /// Load configuration from the process environment.
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
        let data_dir = std::env::var(DATA_DIR_ENV).map_or_else(
            |_| PathBuf::from(DEFAULT_DATA_DIR),
            PathBuf::from,
        );
        Self {
            bind_addr,
            data_dir,
        }
    }

    /// Full path of the `SQLite` database file.
    #[must_use]
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join(SQLITE_FILENAME)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: DEFAULT_BIND_ADDR
                .parse()
                .expect("hard-coded default must parse"),
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
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

    #[test]
    fn db_path_joins_data_dir() {
        let cfg = ServerConfig::default();
        assert!(cfg.db_path().ends_with("server.sqlite"));
    }
}
