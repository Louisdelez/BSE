//! Authentication HTTP handlers : login and register.
//!
//! Returns JSON bodies with a stable shape that the desktop client
//! consumes. Errors map to standard HTTP statuses.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bse_auth::{Claims, TokenType};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::state::AppState;

/// Request body for `POST /api/auth/login`.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Login email.
    pub email: String,
    /// Plaintext password.
    pub password: String,
}

/// Request body for `POST /api/auth/register`.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// Login email.
    pub email: String,
    /// Plaintext password.
    pub password: String,
    /// Display name advertised to peers.
    pub display_name: String,
}

/// Response body for both login and register on success.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// Server-issued user id (UUID v7).
    pub user_id: String,
    /// Display name (mirrors the user record).
    pub display_name: String,
    /// JWT access token.
    pub access_token: String,
    /// JWT refresh token.
    pub refresh_token: String,
    /// Unix timestamp (seconds) at which the access token expires.
    pub access_expires_at: u64,
}

/// Error body returned on failure.
#[derive(Debug, Serialize)]
pub struct AuthError {
    /// Stable machine-readable code (e.g. `"invalid_credentials"`).
    pub code: &'static str,
    /// Human-readable message for the UI.
    pub message: String,
}

fn err(code: &'static str, message: impl Into<String>, status: StatusCode) -> Response {
    let body = Json(AuthError {
        code,
        message: message.into(),
    });
    (status, body).into_response()
}

/// `POST /api/auth/login`.
pub async fn login(State(app): State<AppState>, Json(req): Json<LoginRequest>) -> Response {
    match app.users.verify_credentials(&req.email, &req.password) {
        Ok(Some(user)) => issue_tokens(&app, &user.id.to_string(), &user.email, &user.display_name),
        Ok(None) => err(
            "invalid_credentials",
            "Email or password is incorrect.",
            StatusCode::UNAUTHORIZED,
        ),
        Err(e) => {
            warn!(target: "bse::server::auth", error = %e, "verify_credentials failed");
            err(
                "internal_error",
                "Authentication backend failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

/// `POST /api/auth/register`.
pub async fn register(State(app): State<AppState>, Json(req): Json<RegisterRequest>) -> Response {
    if req.password.len() < 8 {
        return err(
            "weak_password",
            "Password must be at least 8 characters.",
            StatusCode::BAD_REQUEST,
        );
    }
    match app
        .users
        .register(&req.email, &req.display_name, &req.password)
    {
        Ok(Some(id)) => issue_tokens(&app, &id.to_string(), &req.email, &req.display_name),
        Ok(None) => err(
            "email_taken",
            "An account with this email already exists.",
            StatusCode::CONFLICT,
        ),
        Err(e) => {
            warn!(target: "bse::server::auth", error = %e, "register failed");
            err(
                "internal_error",
                "Registration backend failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

fn issue_tokens(app: &AppState, sub: &str, email: &str, display_name: &str) -> Response {
    let access = match app
        .jwt
        .issue(sub, Some(email), Some(display_name), TokenType::Access)
    {
        Ok(t) => t,
        Err(e) => {
            warn!(target: "bse::server::auth", error = %e, "issue access token failed");
            return err(
                "internal_error",
                "Token issuance failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };
    let refresh = match app
        .jwt
        .issue(sub, Some(email), Some(display_name), TokenType::Refresh)
    {
        Ok(t) => t,
        Err(e) => {
            warn!(target: "bse::server::auth", error = %e, "issue refresh token failed");
            return err(
                "internal_error",
                "Token issuance failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };
    // Pull `exp` from the freshly-issued access token so the client
    // does not need to know our TTL constants.
    let access_expires_at = match app.jwt.verify(&access) {
        Ok(Claims { exp, .. }) => exp,
        Err(_) => 0,
    };

    let body = AuthResponse {
        user_id: sub.to_string(),
        display_name: display_name.to_string(),
        access_token: access,
        refresh_token: refresh,
        access_expires_at,
    };
    Json(body).into_response()
}
