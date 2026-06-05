//! Room picker dialog (v021).
//!
//! Lists the rooms the signed-in user is a member of and lets them
//! either join one or create a new one. All HTTP calls are blocking
//! and run on a background thread ; the dialog drains the results
//! through an `mpsc::Receiver` so the egui thread never stalls.

use std::sync::mpsc;

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
    #[allow(clippy::too_many_lines)]
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

        let mut chosen: Option<String> = None;
        egui::Window::new("Choose a room")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(380.0);
                ui.label("Pick a room to enter, or create a new one.");
                ui.add_space(6.0);

                ui.heading("Your rooms");
                if self.rooms.is_empty() && !self.busy() {
                    ui.small("No rooms yet — create your first one below.");
                }
                let rooms = self.rooms.clone();
                let mut invite_clicked: Option<String> = None;
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        for r in &rooms {
                            ui.horizontal(|ui| {
                                ui.label(format!("{} ({})", r.name, r.role));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("Join").clicked() {
                                            chosen = Some(r.id.clone());
                                        }
                                        if r.role == "owner"
                                            && ui.small_button("Invite").clicked()
                                        {
                                            invite_clicked = Some(r.id.clone());
                                        }
                                    },
                                );
                            });
                            if self.invite_target.as_deref() == Some(r.id.as_str()) {
                                ui.indent("invite_form", |ui| {
                                    ui.label(format!("Invite a member to {}", r.name));
                                    ui.add_enabled(
                                        !self.busy(),
                                        egui::TextEdit::singleline(&mut self.invite_email)
                                            .hint_text("email@example.com")
                                            .desired_width(f32::INFINITY),
                                    );
                                    ui.horizontal(|ui| {
                                        let send = ui
                                            .add_enabled(
                                                !self.busy()
                                                    && !self.invite_email.trim().is_empty(),
                                                egui::Button::new("Send invite"),
                                            )
                                            .clicked();
                                        if send {
                                            let email = self.invite_email.trim().to_string();
                                            self.kick_off_invite(
                                                server_http_base,
                                                access_token,
                                                &r.id,
                                                &email,
                                            );
                                        }
                                        if ui.small_button("Close").clicked() {
                                            self.invite_target = None;
                                            self.invite_email.clear();
                                            self.invite_notice.clear();
                                        }
                                    });
                                    if !self.invite_notice.is_empty() {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(0x00, 0xB4, 0x73),
                                            &self.invite_notice,
                                        );
                                    }
                                });
                            }
                        }
                    });
                if let Some(id) = invite_clicked {
                    self.invite_target = Some(id);
                    self.invite_notice.clear();
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);
                ui.heading("Create a room");
                ui.add_enabled(
                    !self.busy(),
                    egui::TextEdit::singleline(&mut self.new_name)
                        .hint_text("Room name")
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let create = ui
                        .add_enabled(
                            !self.busy() && !self.new_name.trim().is_empty(),
                            egui::Button::new("Create"),
                        )
                        .clicked();
                    if create {
                        let name = self.new_name.trim().to_string();
                        self.kick_off_create(server_http_base, access_token, &name);
                    }
                    let refresh = ui
                        .add_enabled(!self.busy(), egui::Button::new("Refresh"))
                        .clicked();
                    if refresh {
                        self.kick_off_fetch(server_http_base, access_token);
                    }
                    if self.busy() {
                        ui.spinner();
                    }
                });

                if !self.error.is_empty() {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_rgb(0xE6, 0x3A, 0x46), &self.error);
                }
            });
        chosen
    }
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
