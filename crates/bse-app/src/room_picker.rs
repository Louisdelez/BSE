//! Room picker dialog (v021).
//!
//! Lists the rooms the signed-in user is a member of and lets them
//! either join one or create a new one. All HTTP calls are blocking
//! and run on a background thread ; the dialog drains the results
//! through an `mpsc::Receiver` so the egui thread never stalls.

use std::sync::mpsc;

use bse_ui::{Card, Modal, PillButton, theme::colors, theme::typography};
use eframe::egui;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Clone, Debug, Deserialize)]
struct RoomResponse {
    id: String,
    name: String,
    role: String,
}

/// One row in the picker.
#[derive(Clone, Debug)]
pub struct RoomEntry {
    /// Room id used in the WS URL.
    pub id: String,
    /// Display name.
    pub name: String,
    /// `"owner"` or `"member"`.
    pub role: String,
}

impl From<RoomResponse> for RoomEntry {
    fn from(r: RoomResponse) -> Self {
        Self {
            id: r.id,
            name: r.name,
            role: r.role,
        }
    }
}

#[derive(Debug, Serialize)]
struct CreateRoomPayload<'a> {
    name: &'a str,
}

#[derive(Debug, Serialize)]
struct InvitePayload<'a> {
    email: &'a str,
}

/// Background-task result : either an updated list, or an error string.
type FetchResult = Result<Vec<RoomEntry>, String>;
type CreateResult = Result<RoomEntry, String>;
type InviteResult = Result<(), String>;

/// In-memory state of the picker.
#[derive(Default)]
pub struct RoomPicker {
    /// Latest known room list.
    pub rooms: Vec<RoomEntry>,
    /// User input for the "create" form.
    pub new_name: String,
    /// Last error shown to the user.
    pub error: String,
    /// Room id currently expanded for the "invite a member" form.
    invite_target: Option<String>,
    /// User input for the invite-email field.
    invite_email: String,
    /// Confirmation flash to show after a successful invite.
    invite_notice: String,
    /// Pending list-fetch result (background thread).
    fetch_rx: Option<mpsc::Receiver<FetchResult>>,
    /// Pending create-room result.
    create_rx: Option<mpsc::Receiver<CreateResult>>,
    /// Pending invite result.
    invite_rx: Option<mpsc::Receiver<InviteResult>>,
    /// `true` once an initial fetch has been kicked off — used to
    /// auto-refresh on first show without re-fetching every frame.
    initial_fetch_started: bool,
}

impl RoomPicker {
    /// Empty picker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset internal state. Called on sign-out.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Drain background results and integrate them.
    fn poll(&mut self) {
        if let Some(rx) = self.fetch_rx.as_ref() {
            match rx.try_recv() {
                Ok(Ok(list)) => {
                    self.rooms = list;
                    self.error.clear();
                    self.fetch_rx = None;
                }
                Ok(Err(msg)) => {
                    self.error = msg;
                    self.fetch_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => self.fetch_rx = None,
            }
        }
        if let Some(rx) = self.create_rx.as_ref() {
            match rx.try_recv() {
                Ok(Ok(entry)) => {
                    self.rooms.insert(0, entry);
                    self.new_name.clear();
                    self.error.clear();
                    self.create_rx = None;
                }
                Ok(Err(msg)) => {
                    self.error = msg;
                    self.create_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => self.create_rx = None,
            }
        }
        if let Some(rx) = self.invite_rx.as_ref() {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    self.invite_notice = format!(
                        "Invited {} to the room.",
                        self.invite_email.trim()
                    );
                    self.invite_email.clear();
                    self.error.clear();
                    self.invite_rx = None;
                }
                Ok(Err(msg)) => {
                    self.error = msg;
                    self.invite_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => self.invite_rx = None,
            }
        }
    }

    fn kick_off_fetch(&mut self, server_http_base: &str, access_token: &str) {
        if self.fetch_rx.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let base = server_http_base.to_string();
        let token = access_token.to_string();
        std::thread::Builder::new()
            .name("bse-rooms-fetch".into())
            .spawn(move || {
                let _ = tx.send(fetch_rooms(&base, &token));
            })
            .ok();
        self.fetch_rx = Some(rx);
    }

    fn kick_off_create(&mut self, server_http_base: &str, access_token: &str, name: &str) {
        if self.create_rx.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let base = server_http_base.to_string();
        let token = access_token.to_string();
        let name = name.to_string();
        std::thread::Builder::new()
            .name("bse-rooms-create".into())
            .spawn(move || {
                let _ = tx.send(create_room(&base, &token, &name));
            })
            .ok();
        self.create_rx = Some(rx);
    }

    fn kick_off_invite(
        &mut self,
        server_http_base: &str,
        access_token: &str,
        room_id: &str,
        email: &str,
    ) {
        if self.invite_rx.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let base = server_http_base.to_string();
        let token = access_token.to_string();
        let room_id = room_id.to_string();
        let email = email.to_string();
        std::thread::Builder::new()
            .name("bse-rooms-invite".into())
            .spawn(move || {
                let _ = tx.send(invite_member(&base, &token, &room_id, &email));
            })
            .ok();
        self.invite_rx = Some(rx);
    }

    /// `true` if a background HTTP call is in flight.
    fn busy(&self) -> bool {
        self.fetch_rx.is_some() || self.create_rx.is_some() || self.invite_rx.is_some()
    }

    /// Render the picker as a modal window. Returns the picked room id
    /// when the user clicks "Join" on a row.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        server_http_base: &str,
        access_token: &str,
    ) -> Option<String> {
        self.poll();
        if !self.initial_fetch_started {
            self.initial_fetch_started = true;
            self.kick_off_fetch(server_http_base, access_token);
        }

        Modal::new("room_picker")
            .title("Choose a room")
            .min_width(480.0)
            .max_width(560.0)
            .show(ctx, |ui| self.body(ui, server_http_base, access_token))
            .inner
    }

    #[allow(clippy::too_many_lines)]
    fn body(
        &mut self,
        ui: &mut egui::Ui,
        server_http_base: &str,
        access_token: &str,
    ) -> Option<String> {
        let mut chosen: Option<String> = None;

        ui.label(
            egui::RichText::new("Pick a room to enter, or create a new one.")
                .color(colors::SLATE)
                .font(typography::size::body_md()),
        );
        ui.add_space(20.0);

        section_heading(ui, "Your rooms");
        ui.add_space(8.0);

        if self.rooms.is_empty() && !self.busy() {
            ui.label(
                egui::RichText::new("No rooms yet — create your first one below.")
                    .color(colors::STEEL)
                    .font(typography::size::body_sm()),
            );
            ui.add_space(8.0);
        }

        let rooms = self.rooms.clone();
        let mut invite_clicked: Option<String> = None;
        egui::ScrollArea::vertical()
            .max_height(280.0)
            .show(ui, |ui| {
                for r in &rooms {
                    self.render_room_row(ui, r, &mut chosen, &mut invite_clicked);
                    if self.invite_target.as_deref() == Some(r.id.as_str()) {
                        self.render_invite_form(ui, r, server_http_base, access_token);
                    }
                    ui.add_space(8.0);
                }
            });
        if let Some(id) = invite_clicked {
            self.invite_target = Some(id);
            self.invite_notice.clear();
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);

        section_heading(ui, "Create a room");
        ui.add_space(8.0);
        ui.add_enabled(
            !self.busy(),
            egui::TextEdit::singleline(&mut self.new_name)
                .hint_text("Room name")
                .desired_width(f32::INFINITY)
                .margin(egui::vec2(12.0, 10.0))
                .font(typography::size::body_md()),
        );
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let create_clicked = ui
                .add(
                    PillButton::primary("Create")
                        .enabled(!self.busy() && !self.new_name.trim().is_empty())
                        .min_size(egui::vec2(0.0, 40.0))
                        .id_source("rooms_create"),
                )
                .clicked();
            if create_clicked {
                let name = self.new_name.trim().to_string();
                self.kick_off_create(server_http_base, access_token, &name);
            }
            ui.add_space(8.0);
            if ui
                .add(
                    PillButton::secondary("Refresh")
                        .enabled(!self.busy())
                        .min_size(egui::vec2(0.0, 40.0))
                        .id_source("rooms_refresh"),
                )
                .clicked()
            {
                self.kick_off_fetch(server_http_base, access_token);
            }
            if self.busy() {
                ui.add_space(8.0);
                ui.spinner();
            }
        });

        if !self.error.is_empty() {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(&self.error)
                    .color(colors::ERROR_TEXT)
                    .font(typography::size::body_sm()),
            );
        }

        chosen
    }

    #[allow(clippy::unused_self)] // kept on Self for symmetry with the other render_* methods
    fn render_room_row(
        &mut self,
        ui: &mut egui::Ui,
        room: &RoomEntry,
        chosen: &mut Option<String>,
        invite_clicked: &mut Option<String>,
    ) {
        Card::base().padding(16.0).radius(12.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&room.name)
                            .color(colors::INK)
                            .font(typography::size::heading_5())
                            .strong(),
                    );
                    ui.add_space(2.0);
                    let (role_label, role_color) = if room.role == "owner" {
                        ("Owner", colors::SUCCESS)
                    } else {
                        ("Member", colors::STEEL)
                    };
                    ui.label(
                        egui::RichText::new(role_label)
                            .color(role_color)
                            .font(typography::size::caption())
                            .strong(),
                    );
                });
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui
                            .add(
                                PillButton::primary("Join")
                                    .min_size(egui::vec2(70.0, 36.0))
                                    .id_source(&format!("join-{}", room.id)),
                            )
                            .clicked()
                        {
                            *chosen = Some(room.id.clone());
                        }
                        if room.role == "owner" {
                            ui.add_space(6.0);
                            if ui
                                .add(
                                    PillButton::secondary("Invite")
                                        .min_size(egui::vec2(72.0, 36.0))
                                        .id_source(&format!("invite-{}", room.id)),
                                )
                                .clicked()
                            {
                                *invite_clicked = Some(room.id.clone());
                            }
                        }
                    },
                );
            });
        });
    }

    fn render_invite_form(
        &mut self,
        ui: &mut egui::Ui,
        room: &RoomEntry,
        server_http_base: &str,
        access_token: &str,
    ) {
        ui.add_space(4.0);
        Card::base()
            .padding(16.0)
            .radius(12.0)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!("Invite a member to {}", room.name))
                        .color(colors::SLATE)
                        .font(typography::size::body_sm())
                        .strong(),
                );
                ui.add_space(8.0);
                ui.add_enabled(
                    !self.busy(),
                    egui::TextEdit::singleline(&mut self.invite_email)
                        .hint_text("email@example.com")
                        .desired_width(f32::INFINITY)
                        .margin(egui::vec2(12.0, 10.0))
                        .font(typography::size::body_md()),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            PillButton::primary("Send invite")
                                .enabled(
                                    !self.busy() && !self.invite_email.trim().is_empty(),
                                )
                                .min_size(egui::vec2(0.0, 36.0))
                                .id_source(&format!("invite-send-{}", room.id)),
                        )
                        .clicked()
                    {
                        let email = self.invite_email.trim().to_string();
                        self.kick_off_invite(server_http_base, access_token, &room.id, &email);
                    }
                    ui.add_space(8.0);
                    if ui
                        .add(
                            PillButton::ghost("Close")
                                .min_size(egui::vec2(0.0, 36.0))
                                .id_source(&format!("invite-close-{}", room.id)),
                        )
                        .clicked()
                    {
                        self.invite_target = None;
                        self.invite_email.clear();
                        self.invite_notice.clear();
                    }
                });
                if !self.invite_notice.is_empty() {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(&self.invite_notice)
                            .color(colors::SUCCESS)
                            .font(typography::size::body_sm()),
                    );
                }
            });
    }
}

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .color(colors::INK)
            .font(typography::size::heading_5())
            .strong(),
    );
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())
}

fn fetch_rooms(server_http_base: &str, access_token: &str) -> FetchResult {
    let url = format!("{server_http_base}/api/rooms");
    let client = http_client()?;
    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .map_err(|e| format!("Connection failed : {e}"))?;
    let status = resp.status();
    if status.is_success() {
        let rows: Vec<RoomResponse> = resp.json().map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(RoomEntry::from).collect())
    } else {
        let msg = resp
            .text()
            .unwrap_or_else(|_| format!("Listing failed (HTTP {status})"));
        warn!(target: "bse::rooms", %status, "fetch_rooms failed");
        Err(msg)
    }
}

fn invite_member(
    server_http_base: &str,
    access_token: &str,
    room_id: &str,
    email: &str,
) -> InviteResult {
    let url = format!("{server_http_base}/api/rooms/{room_id}/members");
    let client = http_client()?;
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&InvitePayload { email })
        .send()
        .map_err(|e| format!("Connection failed : {e}"))?;
    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        let msg = resp
            .text()
            .unwrap_or_else(|_| format!("Invite failed (HTTP {status})"));
        Err(msg)
    }
}

fn create_room(server_http_base: &str, access_token: &str, name: &str) -> CreateResult {
    let url = format!("{server_http_base}/api/rooms");
    let client = http_client()?;
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&CreateRoomPayload { name })
        .send()
        .map_err(|e| format!("Connection failed : {e}"))?;
    let status = resp.status();
    if status.is_success() {
        let row: RoomResponse = resp.json().map_err(|e| e.to_string())?;
        Ok(row.into())
    } else {
        let msg = resp
            .text()
            .unwrap_or_else(|_| format!("Create failed (HTTP {status})"));
        Err(msg)
    }
}
