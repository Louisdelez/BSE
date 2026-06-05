//! Login dialog : prompts for email / password and exchanges them
//! against a JWT via `POST /api/auth/login`.
//!
//! The dialog is shown as a floating modal `egui::Window` when the
//! [`bse_auth::SessionState`] is signed-out. Once login succeeds, the
//! session is persisted in `SqliteStorage` under [`SESSION_KEY`] so
//! subsequent launches skip the prompt until the access token expires.

use bse_auth::SessionState;
use bse_storage::{LocalStorage, SqliteStorage};
use bse_types::UserId;
use eframe::egui;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// `SqliteStorage` key under which the JSON-encoded session is stored.
pub const SESSION_KEY: &str = "session";

#[derive(Debug, Serialize)]
struct LoginPayload<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    user_id: String,
    display_name: String,
    access_token: String,
    refresh_token: String,
    access_expires_at: u64,
}

#[derive(Debug, Deserialize)]
struct AuthError {
    #[allow(dead_code)]
    code: String,
    message: String,
}

/// In-memory state of the login form.
pub struct LoginForm {
    /// Email field buffer.
    pub email: String,
    /// Password field buffer.
    pub password: String,
    /// Last error message to display to the user (empty when none).
    pub error: String,
    /// Whether a request is currently in flight.
    pub busy: bool,
}

impl Default for LoginForm {
    fn default() -> Self {
        Self {
            email: "demo@bse.app".to_string(),
            password: String::new(),
            error: String::new(),
            busy: false,
        }
    }
}

/// Read the persisted session from `SqliteStorage`, if any.
pub fn load_session(storage: &SqliteStorage) -> Option<SessionState> {
    match storage.load_snapshot(SESSION_KEY) {
        Ok(Some(bytes)) => match serde_json::from_slice::<PersistedSession>(&bytes) {
            Ok(p) => Some(p.into()),
            Err(err) => {
                warn!(target: "bse::auth", error = %err, "stored session is malformed, ignoring");
                None
            }
        },
        Ok(None) => None,
        Err(err) => {
            warn!(target: "bse::auth", error = %err, "load_session failed");
            None
        }
    }
}

/// Persist `session` to `SqliteStorage` (no-op for `SignedOut`).
pub fn persist_session(storage: &mut SqliteStorage, session: &SessionState) {
    let persisted: PersistedSession = match session {
        SessionState::SignedOut => return,
        SessionState::SignedIn {
            user_id,
            display_name,
            access_token,
            refresh_token,
            access_expires_at,
        } => PersistedSession {
            user_id: *user_id,
            display_name: display_name.clone(),
            access_token: access_token.clone(),
            refresh_token: refresh_token.clone(),
            access_expires_at: *access_expires_at,
        },
    };
    let Ok(bytes) = serde_json::to_vec(&persisted) else {
        return;
    };
    if let Err(err) = storage.save_snapshot(SESSION_KEY, &bytes) {
        warn!(target: "bse::auth", error = %err, "persist_session failed");
    }
}

/// Render the login modal and, on Submit, run a blocking HTTP POST to
/// `{server_url}/api/auth/login`. Returns `Some(SessionState)` on
/// success, `None` while the user is still typing or after a failure.
///
/// `server_url` is the HTTP(S) base of the server (not the WebSocket
/// URL). When the app has only a `ws://` URL, swap the scheme : e.g.
/// `ws://localhost:8080` → `http://localhost:8080`.
pub fn show_modal(
    ctx: &egui::Context,
    form: &mut LoginForm,
    server_url: &str,
) -> Option<SessionState> {
    let mut result = None;
    egui::Window::new("Sign in to BSE")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, -20.0])
        .show(ctx, |ui| {
            ui.set_min_width(320.0);
            ui.add_space(4.0);
            ui.label("Connect to your BSE server.");
            ui.add_space(8.0);

            ui.label("Email");
            ui.add_enabled(
                !form.busy,
                egui::TextEdit::singleline(&mut form.email).desired_width(f32::INFINITY),
            );

            ui.label("Password");
            ui.add_enabled(
                !form.busy,
                egui::TextEdit::singleline(&mut form.password)
                    .password(true)
                    .desired_width(f32::INFINITY),
            );

            if !form.error.is_empty() {
                ui.add_space(4.0);
                ui.colored_label(egui::Color32::from_rgb(0xE6, 0x3A, 0x46), &form.error);
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let submit = ui
                    .add_enabled(!form.busy, egui::Button::new("Sign in"))
                    .clicked()
                    || (!form.busy && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                if submit {
                    // The HTTP call below is blocking, so this branch
                    // runs synchronously ; no need to flip `busy` (it
                    // would just be reset at the bottom of the same
                    // frame).
                    form.error.clear();
                    match try_login(server_url, &form.email, &form.password) {
                        Ok(state) => {
                            form.password.clear();
                            result = Some(state);
                        }
                        Err(msg) => {
                            form.error = msg;
                        }
                    }
                }
                if form.busy {
                    ui.spinner();
                }
            });

            ui.add_space(4.0);
            ui.small("Demo : demo@bse.app / demo1234");
        });
    result
}

fn try_login(server_url: &str, email: &str, password: &str) -> Result<SessionState, String> {
    let url = format!("{}/api/auth/login", normalize_base(server_url));
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .json(&LoginPayload { email, password })
        .send()
        .map_err(|e| format!("Connection failed : {e}"))?;
    let status = resp.status();
    if status.is_success() {
        let body: AuthResponse = resp.json().map_err(|e| e.to_string())?;
        let user_id = body
            .user_id
            .parse::<UserId>()
            .map_err(|e| format!("Invalid user id from server : {e}"))?;
        info!(target: "bse::auth", user = %body.display_name, "signed in");
        Ok(SessionState::SignedIn {
            user_id,
            display_name: body.display_name,
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            access_expires_at: body.access_expires_at,
        })
    } else {
        let msg = resp
            .json::<AuthError>()
            .map_or_else(|_| format!("Login failed (HTTP {status})"), |e| e.message);
        Err(msg)
    }
}

/// Replace `ws://` / `wss://` with `http://` / `https://` and trim
/// any trailing slash, so the result is a clean HTTP base.
fn normalize_base(url: &str) -> String {
    let s = url
        .replacen("ws://", "http://", 1)
        .replacen("wss://", "https://", 1);
    s.trim_end_matches('/').to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSession {
    user_id: UserId,
    display_name: String,
    access_token: String,
    refresh_token: String,
    access_expires_at: u64,
}

impl From<PersistedSession> for SessionState {
    fn from(p: PersistedSession) -> Self {
        Self::SignedIn {
            user_id: p.user_id,
            display_name: p.display_name,
            access_token: p.access_token,
            refresh_token: p.refresh_token,
            access_expires_at: p.access_expires_at,
        }
    }
}
