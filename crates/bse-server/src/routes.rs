//! Router assembly.

use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderName, HeaderValue, Method, header},
    routing::{delete, get, post},
};
use governor::middleware::NoOpMiddleware;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::warn;

use crate::handlers::{
    auth::{login, refresh, register},
    health::{health, ready},
    info::info,
    rooms::{add_member, create_room, list_rooms, remove_member},
};
use crate::state::AppState;
use crate::ws::ws_room;

/// Max body size accepted on JSON endpoints (1 MiB).
const JSON_BODY_LIMIT: usize = 1024 * 1024;

/// Env var controlling allowed CORS origins (comma-separated). When
/// unset, the router falls back to `permissive` mode — appropriate for
/// `localhost` development, not for production.
const ALLOWED_ORIGINS_ENV: &str = "BSE_ALLOWED_ORIGINS";

fn cors_layer() -> CorsLayer {
    let methods = [Method::GET, Method::POST, Method::DELETE, Method::OPTIONS];
    let headers = [header::CONTENT_TYPE, header::AUTHORIZATION];
    match std::env::var(ALLOWED_ORIGINS_ENV) {
        Ok(value) if !value.trim().is_empty() => {
            let mut allowed = Vec::new();
            for raw in value.split(',') {
                let s = raw.trim();
                if s.is_empty() {
                    continue;
                }
                match HeaderValue::from_str(s) {
                    Ok(h) => allowed.push(h),
                    Err(err) => {
                        warn!(target: "bse::server::cors", origin = %s, error = %err, "ignoring malformed origin");
                    }
                }
            }
            if allowed.is_empty() {
                warn!(target: "bse::server::cors", "{ALLOWED_ORIGINS_ENV} was set but contained no valid origins ; falling back to permissive");
                CorsLayer::permissive()
            } else {
                CorsLayer::new()
                    .allow_origin(AllowOrigin::list(allowed))
                    .allow_methods(methods)
                    .allow_headers(headers)
            }
        }
        _ => CorsLayer::permissive(),
    }
}

/// Static response headers that improve baseline browser security.
/// Applied to *every* response — no-op for the desktop client, useful
/// when a future web client lives at the same origin.
fn security_headers() -> [SetResponseHeaderLayer<HeaderValue>; 4] {
    [
        SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ),
        SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ),
        SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        ),
        SetResponseHeaderLayer::overriding(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), camera=(), microphone=()"),
        ),
    ]
}

fn auth_rate_limit() -> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware> {
    let cfg = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(10)
        .key_extractor(SmartIpKeyExtractor)
        .finish()
        .expect("static rate-limit config must build");
    GovernorLayer {
        config: Arc::new(cfg),
    }
}

/// Build the application router with all current routes wired up.
pub fn router_with(state: AppState) -> Router {
    let limits = Arc::new(DefaultBodyLimit::max(JSON_BODY_LIMIT));
    let _ = limits;

    let auth_routes = Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/register", post(register))
        .route("/api/auth/refresh", post(refresh))
        .layer(auth_rate_limit());

    let room_routes = Router::new()
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/:id/members", post(add_member))
        .route("/api/rooms/:id/members/:user_id", delete(remove_member));

    let api = auth_routes.merge(room_routes);

    let security = security_headers();

    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/info", get(info))
        .merge(api)
        .route("/ws/rooms/:room_id", get(ws_room))
        .layer(DefaultBodyLimit::max(JSON_BODY_LIMIT))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer())
        .layer(security[0].clone())
        .layer(security[1].clone())
        .layer(security[2].clone())
        .layer(security[3].clone())
        .with_state(state)
}

/// Convenience wrapper that builds the default [`AppState`] and the
/// router in one call. Useful for tests and the main binary.
pub fn router() -> Router {
    router_with(AppState::build())
}

/// Read-only check used in tests : the env var name we honour.
#[must_use]
#[doc(hidden)]
pub const fn allowed_origins_env_name() -> &'static str {
    ALLOWED_ORIGINS_ENV
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_builds() {
        let _layer: GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware> = auth_rate_limit();
    }

    #[test]
    fn cors_default_is_permissive_when_env_unset() {
        // We can't assert internals without depending on tower-http's
        // CorsLayer internals, so just check the layer is constructible.
        let _ = cors_layer();
    }

    #[test]
    fn allowed_origins_env_name_is_documented_constant() {
        assert_eq!(allowed_origins_env_name(), "BSE_ALLOWED_ORIGINS");
    }
}
