//! Shared application state injected into every axum handler.
//!
//! Holds the persistent [`ServerStore`], the [`UserStore`] wrapper, the
//! [`JwtConfig`] and the [`RoomManager`].

use std::sync::Arc;

use bse_auth::{DEFAULT_ACCESS_TTL, DEFAULT_REFRESH_TTL, JwtConfig};
use tracing::{error, info};

use crate::config::ServerConfig;
use crate::rooms::RoomManager;
use crate::store::ServerStore;
use crate::users::UserStore;

/// Shared state injected into handlers as `axum::extract::State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// SQLite-backed server store (users, room snapshots).
    pub store: Arc<ServerStore>,
    /// User-facing wrapper over [`ServerStore`].
    pub users: UserStore,
    /// JWT configuration (secret, issuer, TTLs).
    pub jwt: Arc<JwtConfig>,
    /// Per-room broadcast registry (v010.2).
    pub rooms: RoomManager,
}

impl AppState {
    /// Build the state from the runtime config.
    ///
    /// Opens (or creates) the `SQLite` database at
    /// `{cfg.data_dir}/server.sqlite`, runs schema migrations, seeds a
    /// demo user if and only if the `users` table is empty.
    ///
    /// The JWT secret comes from the `BSE_JWT_SECRET` env var if set,
    /// otherwise a process-local pseudo-random secret is generated.
    /// The latter is fine for development but unsuitable for
    /// multi-instance deployments — set `BSE_JWT_SECRET` in production.
    #[must_use]
    pub fn from_config(cfg: &ServerConfig) -> Self {
        let store = match ServerStore::open(&cfg.db_path()) {
            Ok(s) => Arc::new(s),
            Err(err) => {
                error!(error = %err, path = %cfg.db_path().display(), "failed to open server store ; falling back to in-memory");
                Arc::new(ServerStore::in_memory().expect("in-memory store always opens"))
            }
        };
        let users = UserStore::new(Arc::clone(&store));
        if let Err(err) = users.seed_if_empty("demo@bse.app", "Demo", "demo1234") {
            error!(target: "bse::server::state", error = %err, "demo seed failed");
        } else {
            info!(target: "bse::server::state", "demo user seeded if empty");
        }

        let secret = std::env::var("BSE_JWT_SECRET")
            .map_or_else(|_| derive_process_local_secret(), String::into_bytes);
        let jwt = JwtConfig {
            secret,
            issuer: "bse".to_string(),
            access_ttl: DEFAULT_ACCESS_TTL,
            refresh_ttl: DEFAULT_REFRESH_TTL,
        };

        let rooms = RoomManager::new(Arc::clone(&store));

        Self {
            store,
            users,
            jwt: Arc::new(jwt),
            rooms,
        }
    }

    /// Convenience builder using [`ServerConfig::from_env`].
    #[must_use]
    pub fn build() -> Self {
        Self::from_config(&ServerConfig::from_env())
    }
}

/// Build a 64-byte pseudo-random secret tied to the current process.
/// Sufficient for dev ; insufficient for prod (use `BSE_JWT_SECRET`).
fn derive_process_local_secret() -> Vec<u8> {
    let pid = u64::from(std::process::id());
    let mut bytes = vec![0u8; 64];
    for (i, b) in bytes.iter_mut().enumerate() {
        let n = pid.wrapping_mul(2_654_435_761).wrapping_add(i as u64) & 0xFF;
        // `n` is masked to a u8 range, so the truncation is exact.
        #[allow(clippy::cast_possible_truncation)]
        let v = n as u8;
        *b = v;
    }
    bytes
}
