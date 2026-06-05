//! Login / sign-up modal and session persistence (v016.1, extended v020).
//!
//! Displays a floating `egui::Window` when the user is signed out. The
//! form has two modes :
//!
//! - **Sign in** : email + password → `POST /api/auth/login`.
//! - **Sign up** : email + display name + password → `POST /api/auth/register`.
//!
//! On success the resulting [`bse_auth::SessionState`] is persisted in
//! `SqliteStorage` under [`SESSION_KEY`] so subsequent launches skip the
//! prompt. The same module owns the auto-refresh helper used by
//! [`crate::app::BseApp`] to renew access tokens before expiry.

use bse_auth::SessionState;
use bse_storage::{LocalStorage, SqliteStorage};
use bse_types::UserId;
use bse_ui::{Modal, PillButton, theme::colors, theme::typography};
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

#[derive(Debug, Serialize)]
struct RegisterPayload<'a> {
    email: &'a str,
    password: &'a str,
    display_name: &'a str,
}

#[derive(Debug, Serialize)]
struct RefreshPayload<'a> {
    refresh_token: &'a str,
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

/// Which sub-form the modal currently shows.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMode {
    /// Existing-user sign-in.
    #[default]
    SignIn,
    /// New-account registration.
    SignUp,
}

/// In-memory state of the login form.
pub struct LoginForm {
    /// Email field buffer.
    pub email: String,
    /// Password field buffer.
    pub password: String,
    /// Display name field buffer (sign-up only).
    pub display_name: String,
    /// Last error message to display to the user (empty when none).
    pub error: String,
    /// Whether a request is currently in flight.
    pub busy: bool,
    /// Which sub-form is shown.
    pub mode: LoginMode,
}

impl Default for LoginForm {
    fn default() -> Self {
        Self {
            email: "demo@bse.app".to_string(),
            password: String::new(),
            display_name: String::new(),
            error: String::new(),
            busy: false,
            mode: LoginMode::default(),
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

/// Clear any persisted session row. Called on sign-out.
pub fn clear_session(storage: &mut SqliteStorage) {
    // The `LocalStorage` trait does not expose a delete primitive yet ;
    // overwriting with an empty blob makes the next `load_session`
    // discard it as malformed and return `None`.
    if let Err(err) = storage.save_snapshot(SESSION_KEY, &[]) {
        warn!(target: "bse::auth", error = %err, "clear_session failed");
    }
}

/// Render the login / sign-up modal. Returns `Some(SessionState)` on
/// success, `None` otherwise.
pub fn show_modal(
    ctx: &egui::Context,
    form: &mut LoginForm,
    server_url: &str,
) -> Option<SessionState> {
    let title = match form.mode {
        LoginMode::SignIn => "Sign in to BSE",
        LoginMode::SignUp => "Create your BSE account",
    };

    Modal::new("login")
        .title(title)
        .min_width(380.0)
        .max_width(420.0)
        .show(ctx, |ui| render_form(ui, form, server_url))
        .inner
}

#[allow(clippy::too_many_lines)]
fn render_form(
    ui: &mut egui::Ui,
    form: &mut LoginForm,
    server_url: &str,
) -> Option<SessionState> {
    let mut result = None;

    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(match form.mode {
            LoginMode::SignIn => "Welcome back. Sign in to your BSE server.",
            LoginMode::SignUp => "Choose your credentials and start collaborating.",
        })
        .color(colors::SLATE)
        .font(typography::size::body_md()),
    );
    ui.add_space(16.0);

    field_label(ui, "Email");
    ui.add_enabled(
        !form.busy,
        egui::TextEdit::singleline(&mut form.email)
            .desired_width(f32::INFINITY)
            .margin(egui::vec2(12.0, 10.0))
            .font(typography::size::body_md()),
    );

    if matches!(form.mode, LoginMode::SignUp) {
        ui.add_space(12.0);
        field_label(ui, "Display name");
        ui.add_enabled(
            !form.busy,
            egui::TextEdit::singleline(&mut form.display_name)
                .desired_width(f32::INFINITY)
                .margin(egui::vec2(12.0, 10.0))
                .font(typography::size::body_md()),
        );
    }

    ui.add_space(12.0);
    field_label(ui, "Password");
    ui.add_enabled(
        !form.busy,
        egui::TextEdit::singleline(&mut form.password)
            .password(true)
            .desired_width(f32::INFINITY)
            .margin(egui::vec2(12.0, 10.0))
            .font(typography::size::body_md()),
    );

    if !form.error.is_empty() {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(&form.error)
                .color(colors::ERROR_TEXT)
                .font(typography::size::body_sm()),
        );
    }

    ui.add_space(20.0);

    let submit_text = match form.mode {
        LoginMode::SignIn => "Sign in",
        LoginMode::SignUp => "Create account",
    };
    let submit = ui
        .add(
            PillButton::primary(submit_text)
                .enabled(!form.busy)
                .min_size(egui::vec2(0.0, 44.0))
                .id_source("login_submit"),
        )
        .clicked()
        || (!form.busy && ui.input(|i| i.key_pressed(egui::Key::Enter)));

    if submit {
        form.error.clear();
        let res = match form.mode {
            LoginMode::SignIn => try_login(server_url, &form.email, &form.password),
            LoginMode::SignUp => {
                try_register(server_url, &form.email, &form.display_name, &form.password)
            }
        };
        match res {
            Ok(state) => {
                form.password.clear();
                form.display_name.clear();
                result = Some(state);
            }
            Err(msg) => form.error = msg,
        }
    }

    if form.busy {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                egui::RichText::new("Talking to the server…")
                    .color(colors::STEEL)
                    .font(typography::size::body_sm()),
            );
        });
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    let toggle_label = match form.mode {
        LoginMode::SignIn => "Need an account ? Sign up",
        LoginMode::SignUp => "Have an account ? Sign in",
    };
    if ui
        .add(
            PillButton::ghost(toggle_label)
                .min_size(egui::vec2(0.0, 36.0))
                .id_source("login_toggle"),
        )
        .clicked()
    {
        form.mode = match form.mode {
            LoginMode::SignIn => LoginMode::SignUp,
            LoginMode::SignUp => LoginMode::SignIn,
        };
        form.error.clear();
    }

    ui.add_space(4.0);
    let helper = match form.mode {
        LoginMode::SignIn => "Demo : demo@bse.app / demo1234",
        LoginMode::SignUp => "Password must be at least 8 characters.",
    };
    ui.label(
        egui::RichText::new(helper)
            .color(colors::STEEL)
            .font(typography::size::caption()),
    );

    result
}

fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .color(colors::SLATE)
            .font(typography::size::body_sm())
            .strong(),
    );
    ui.add_space(4.0);
}

fn try_login(server_url: &str, email: &str, password: &str) -> Result<SessionState, String> {
    let url = format!("{}/api/auth/login", normalize_base(server_url));
    let client = http_client()?;
    let resp = client
        .post(&url)
        .json(&LoginPayload { email, password })
        .send()
        .map_err(|e| format!("Connection failed : {e}"))?;
    decode_auth_response(resp).map(|body| {
        info!(target: "bse::auth", user = %body.display_name, "signed in");
        body.into()
    })
}

fn try_register(
    server_url: &str,
    email: &str,
    display_name: &str,
    password: &str,
) -> Result<SessionState, String> {
    let url = format!("{}/api/auth/register", normalize_base(server_url));
    let client = http_client()?;
    let resp = client
        .post(&url)
        .json(&RegisterPayload {
            email,
            password,
            display_name,
        })
        .send()
        .map_err(|e| format!("Connection failed : {e}"))?;
    decode_auth_response(resp).map(|body| {
        info!(target: "bse::auth", user = %body.display_name, "signed up");
        body.into()
    })
}

/// Refresh the access token, blocking until completion.
///
/// Used by the desktop app via a background thread (see
/// `BseApp::maybe_refresh_token`). Returns the new session on success
/// or an error message suitable for surfacing in the UI.
pub fn try_refresh(server_url: &str, refresh_token: &str) -> Result<SessionState, String> {
    let url = format!("{}/api/auth/refresh", normalize_base(server_url));
    let client = http_client()?;
    let resp = client
        .post(&url)
        .json(&RefreshPayload { refresh_token })
        .send()
        .map_err(|e| format!("Connection failed : {e}"))?;
    decode_auth_response(resp).map(|body| {
        info!(target: "bse::auth", user = %body.display_name, "session refreshed");
        body.into()
    })
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())
}

fn decode_auth_response(resp: reqwest::blocking::Response) -> Result<AuthResponse, String> {
    let status = resp.status();
    if status.is_success() {
        resp.json::<AuthResponse>().map_err(|e| e.to_string())
    } else {
        let msg = resp
            .json::<AuthError>()
            .map_or_else(|_| format!("Request failed (HTTP {status})"), |e| e.message);
        Err(msg)
    }
}

impl From<AuthResponse> for SessionState {
    fn from(body: AuthResponse) -> Self {
        let user_id = body
            .user_id
            .parse::<UserId>()
            .unwrap_or_else(|_| UserId::new());
        Self::SignedIn {
            user_id,
            display_name: body.display_name,
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            access_expires_at: body.access_expires_at,
        }
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
