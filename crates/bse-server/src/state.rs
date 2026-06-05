//! Shared application state injected into every axum handler.
//!
//! Currently holds the [`UserStore`] and the [`JwtConfig`]. Will grow
//! to include the room manager in v010.2.

use std::sync::Arc;

use bse_auth::{DEFAULT_ACCESS_TTL, DEFAULT_REFRESH_TTL, JwtConfig};

use crate::users::UserStore;

/// Shared state injected into handlers as `axum::extract::State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// In-memory user store.
    pub users: Arc<UserStore>,
    /// JWT configuration (secret, issuer, TTLs).
    pub jwt: Arc<JwtConfig>,
}

impl AppState {
    /// Build the state from the runtime config.
    ///
    /// The JWT secret comes from the `BSE_JWT_SECRET` env var if set,
    /// otherwise a process-local pseudo-random secret is generated.
    /// The latter is fine for development but useless for multi-instance
    /// deployments — set `BSE_JWT_SECRET` in production.
    #[must_use]
    pub fn build() -> Self {
        let secret = std::env::var("BSE_JWT_SECRET")
            .map_or_else(|_| derive_process_local_secret(), String::into_bytes);
        let jwt = JwtConfig {
            secret,
            issuer: "bse".to_string(),
            access_ttl: DEFAULT_ACCESS_TTL,
            refresh_ttl: DEFAULT_REFRESH_TTL,
        };
        let users = UserStore::new();
        // Seed a demo user so first-run testing works without the
        // (not yet exposed via UI) register endpoint.
        let _ = users.seed_if_empty("demo@bse.app", "Demo", "demo1234");
        Self {
            users: Arc::new(users),
            jwt: Arc::new(jwt),
        }
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
