//! Room management endpoints (v021).
//!
//! Routes :
//! - `GET    /api/rooms`                     — list my rooms.
//! - `POST   /api/rooms`                     — create a room, become owner.
//! - `POST   /api/rooms/:id/members`         — invite a user (owner only).
//! - `DELETE /api/rooms/:id/members/:user_id`— kick a member (owner only).
//!
//! All routes expect a `Bearer <access_token>` `Authorization` header
//! and 401 anything that does not verify.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use bse_auth::{Claims, TokenType};
use bse_types::UserId;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::state::AppState;
use crate::store::RoomRole;

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

fn err(code: &'static str, message: impl Into<String>, status: StatusCode) -> Response {
    (
        status,
        Json(ErrorBody {
            code,
            message: message.into(),
        }),
    )
        .into_response()
}

/// Extract the bearer token from `Authorization` and verify it as an
/// access token. Returns the parsed `UserId` of the caller.
#[allow(clippy::result_large_err)] // Response is intentionally large
fn require_user(headers: &HeaderMap, app: &AppState) -> Result<UserId, Response> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| {
            err(
                "missing_token",
                "Authorization: Bearer <token> required.",
                StatusCode::UNAUTHORIZED,
            )
        })?;
    let claims: Claims = app
        .jwt
        .verify(token)
        .map_err(|e| err("invalid_token", e.to_string(), StatusCode::UNAUTHORIZED))?;
    if claims.token_type != TokenType::Access {
        return Err(err(
            "wrong_token_type",
            "Expected an access token.",
            StatusCode::UNAUTHORIZED,
        ));
    }
    claims.sub.parse::<UserId>().map_err(|e| {
        err(
            "invalid_subject",
            format!("token subject is not a user id : {e}"),
            StatusCode::UNAUTHORIZED,
        )
    })
}

/// A room as returned by the list / create endpoints.
#[derive(Debug, Serialize)]
pub struct RoomView {
    /// Room id (used in the WS URL).
    pub id: String,
    /// Display name.
    pub name: String,
    /// User who created the room.
    pub created_by: String,
    /// Caller's role in the room.
    pub role: String,
    /// Unix seconds of creation.
    pub created_at: i64,
}

/// Body of `POST /api/rooms`.
#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    /// Optional explicit id. If absent, a `UUIDv7` is generated.
    #[serde(default)]
    pub id: Option<String>,
    /// Display name. Must be non-empty.
    pub name: String,
}

/// Body of `POST /api/rooms/:id/members`.
#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    /// Email of the user to invite.
    pub email: String,
    /// Role to grant. Defaults to `"member"`.
    #[serde(default)]
    pub role: Option<String>,
}

/// `GET /api/rooms`.
pub async fn list_rooms(State(app): State<AppState>, headers: HeaderMap) -> Response {
    let user_id = match require_user(&headers, &app) {
        Ok(u) => u,
        Err(r) => return r,
    };
    match app.store.list_user_rooms(user_id) {
        Ok(rows) => {
            let views: Vec<RoomView> = rows
                .into_iter()
                .map(|r| RoomView {
                    id: r.room.id,
                    name: r.room.name,
                    created_by: r.room.created_by.to_string(),
                    role: r.role.as_str().to_string(),
                    created_at: r.room.created_at,
                })
                .collect();
            Json(views).into_response()
        }
        Err(e) => {
            warn!(target: "bse::server::rooms", error = %e, "list_user_rooms failed");
            err(
                "internal_error",
                "Room listing failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

/// `POST /api/rooms`.
pub async fn create_room(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateRoomRequest>,
) -> Response {
    let user_id = match require_user(&headers, &app) {
        Ok(u) => u,
        Err(r) => return r,
    };
    let name = req.name.trim();
    if name.is_empty() {
        return err(
            "invalid_name",
            "Room name cannot be empty.",
            StatusCode::BAD_REQUEST,
        );
    }
    let id = req
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| bse_types::UserId::new_v7().to_string());
    match app.store.create_room(&id, name, user_id) {
        Ok(true) => Json(RoomView {
            id: id.clone(),
            name: name.to_string(),
            created_by: user_id.to_string(),
            role: RoomRole::Owner.as_str().to_string(),
            created_at: 0,
        })
        .into_response(),
        Ok(false) => err(
            "room_exists",
            "A room with this id already exists.",
            StatusCode::CONFLICT,
        ),
        Err(e) => {
            warn!(target: "bse::server::rooms", error = %e, "create_room failed");
            err(
                "internal_error",
                "Room creation failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

/// `POST /api/rooms/:id/members`. Owner-only.
pub async fn add_member(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(room_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Response {
    let caller = match require_user(&headers, &app) {
        Ok(u) => u,
        Err(r) => return r,
    };
    if app.store.role_of(&room_id, caller).ok().flatten() != Some(RoomRole::Owner) {
        return err(
            "forbidden",
            "Only the room owner can invite members.",
            StatusCode::FORBIDDEN,
        );
    }
    let target = match app.users.verify_email_only(&req.email) {
        Ok(Some(u)) => u,
        Ok(None) => {
            return err(
                "unknown_user",
                "No account exists with this email.",
                StatusCode::NOT_FOUND,
            );
        }
        Err(e) => {
            warn!(target: "bse::server::rooms", error = %e, "lookup target user failed");
            return err(
                "internal_error",
                "Member lookup failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };
    let role = req
        .role
        .as_deref()
        .and_then(RoomRole::parse)
        .unwrap_or(RoomRole::Member);
    match app.store.add_member(&room_id, target.id, role) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            warn!(target: "bse::server::rooms", error = %e, "add_member failed");
            err(
                "internal_error",
                "Adding member failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

/// `DELETE /api/rooms/:id/members/:user_id`. Owner-only.
pub async fn remove_member(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path((room_id, target_user_id)): Path<(String, String)>,
) -> Response {
    let caller = match require_user(&headers, &app) {
        Ok(u) => u,
        Err(r) => return r,
    };
    if app.store.role_of(&room_id, caller).ok().flatten() != Some(RoomRole::Owner) {
        return err(
            "forbidden",
            "Only the room owner can kick members.",
            StatusCode::FORBIDDEN,
        );
    }
    let target: UserId = match target_user_id.parse() {
        Ok(u) => u,
        Err(e) => {
            return err(
                "invalid_user_id",
                format!("Bad user id : {e}"),
                StatusCode::BAD_REQUEST,
            );
        }
    };
    if target == caller {
        return err(
            "cannot_remove_self",
            "An owner cannot remove themselves.",
            StatusCode::BAD_REQUEST,
        );
    }
    match app.store.remove_member(&room_id, target) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            warn!(target: "bse::server::rooms", error = %e, "remove_member failed");
            err(
                "internal_error",
                "Removing member failed.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}
