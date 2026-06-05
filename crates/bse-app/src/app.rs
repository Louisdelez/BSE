//! Top-level [`eframe::App`] implementation for BSE.
//!
//! Owns the canvas state, the CRDT-backed document, the spatial index,
//! the asset store, the local persistence handle and (optionally) the
//! sync worker thread. Coordinates the layout between toolbar, canvas,
//! and status bar.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use bse_auth::SessionState;
use bse_canvas::CanvasState;
use bse_crdt::{CrdtBackend, YrsBackend};
use bse_spatial::Quadtree;
use bse_storage::{LocalStorage, SqliteStorage};
use bse_sync::{ClientConfig, ConnectionState};
use bse_types::{ElementId, PeerId, Rect as WorldRect, Vec2 as WorldVec2};
use bse_ui::{ConnectionStatus, StatusInfo, ThemeMode, apply_bse_theme, status_bar, toolbar};
use eframe::egui;
use tracing::{info, warn};

use crate::APP_INFO;
use crate::assets::AssetStore;
use crate::canvas;
use crate::login::{self, LoginForm};
use crate::peers::PeerStore;
use crate::project_io;
use crate::room_picker::RoomPicker;
use crate::sync_thread::{SyncCmd, SyncEvent, SyncHandle};

const SPATIAL_HALF_EXTENT: f32 = 1_000_000.0;
const SPATIAL_MAX_ITEMS_PER_LEAF: usize = 16;
const SPATIAL_MAX_DEPTH: u32 = 10;

/// Key used to autosave the current document in `SqliteStorage`.
const AUTOSAVE_KEY: &str = "current-project";
const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(5);

const IMAGE_PIXEL_WARN: u32 = 4096;

/// Refresh the access token when it has less than this many seconds left.
const REFRESH_LEAD_SECS: u64 = 60;

/// Root application state.
pub struct BseApp {
    canvas: CanvasState,
    crdt: YrsBackend,
    spatial: Quadtree<ElementId>,
    assets: AssetStore,
    storage: Option<SqliteStorage>,
    dirty: bool,
    last_save: Instant,
    fps: f32,
    last_frame: Option<Instant>,
    last_visible_count: u32,
    last_element_count: usize,
    /// `Some` if a sync worker thread was spawned at startup
    /// (controlled by the `BSE_SERVER_URL` env var).
    sync: Option<SyncHandle>,
    /// Current state of the WebSocket connection (kept for the status bar).
    connection_state: ConnectionState,
    /// Remote peer cursors / names.
    peers: PeerStore,
    /// Stable identifier of the local peer for awareness messages.
    local_peer_id: PeerId,
    /// Last cursor position emitted to the sync thread (world coords).
    last_emitted_cursor: Option<WorldVec2>,
    /// Current login state. `SignedOut` triggers the login modal.
    session: SessionState,
    /// Backing state for the login form widget.
    login_form: LoginForm,
    /// Cached HTTP base of the server (`ws://` → `http://`). Empty when
    /// no `BSE_SERVER_URL` is configured.
    server_http_base: String,
    /// `Some` while a background token-refresh request is in flight.
    /// Drains a fresh [`SessionState`] (or an error message) on completion.
    refresh_rx: Option<mpsc::Receiver<Result<SessionState, String>>>,
    /// Room picker dialog state (v021).
    room_picker: RoomPicker,
    /// Room currently connected to. `None` until the user picks one
    /// (or `BSE_ROOM` is set, which is treated as an implicit pick).
    current_room: Option<String>,
    /// Whether the picker is currently shown.
    show_picker: bool,
    /// State vector of the last update we broadcast. Used to ship
    /// *incremental* updates rather than full snapshots (v023).
    /// Empty until the first broadcast, which sends a full snapshot.
    last_broadcast_sv: Vec<u8>,
    /// Whether the BSE theme has been applied yet. `apply_bse_theme`
    /// needs an `egui::Context`, which we only get inside `update`, so
    /// it's deferred to the first frame.
    theme_applied: bool,
}

impl Default for BseApp {
    fn default() -> Self {
        Self::new()
    }
}

impl BseApp {
    /// Build a fresh app. Loads the previous session from the local
    /// `SQLite` store if present and, if `BSE_SERVER_URL` is set, spawns
    /// the sync worker thread.
    #[must_use]
    pub fn new() -> Self {
        let bounds = WorldRect::from_min_max(
            WorldVec2::splat(-SPATIAL_HALF_EXTENT),
            WorldVec2::splat(SPATIAL_HALF_EXTENT),
        );
        let mut crdt = YrsBackend::new();
        let storage = open_default_storage();
        if let Some(store) = &storage {
            match store.load_snapshot(AUTOSAVE_KEY) {
                Ok(Some(bytes)) => match crdt.apply_remote_update(&bytes) {
                    Ok(()) => info!(
                        target: "bse::app",
                        elements = crdt.element_count(),
                        "loaded previous session",
                    ),
                    Err(err) => warn!(
                        target: "bse::app",
                        error = %err,
                        "previous snapshot rejected ; starting fresh",
                    ),
                },
                Ok(None) => info!(target: "bse::app", "no previous session, starting fresh"),
                Err(err) => warn!(target: "bse::app", error = %err, "load_snapshot failed"),
            }
        }
        let elements = crdt.element_count();
        let local_peer_id = PeerId::new();
        let server_url = std::env::var("BSE_SERVER_URL").unwrap_or_default();
        let server_http_base = if server_url.is_empty() {
            String::new()
        } else {
            server_url
                .replacen("ws://", "http://", 1)
                .replacen("wss://", "https://", 1)
                .trim_end_matches('/')
                .to_string()
        };
        let session = storage
            .as_ref()
            .and_then(login::load_session)
            .unwrap_or_default();
        // Sync is now spawned lazily once a room is picked ; see
        // `BseApp::ensure_sync_connected`.
        let sync: Option<SyncHandle> = None;
        Self {
            canvas: CanvasState::new(),
            crdt,
            spatial: Quadtree::new(bounds, SPATIAL_MAX_ITEMS_PER_LEAF, SPATIAL_MAX_DEPTH),
            assets: AssetStore::new(),
            storage,
            dirty: false,
            last_save: Instant::now(),
            fps: 0.0,
            last_frame: None,
            last_visible_count: 0,
            last_element_count: elements,
            sync,
            connection_state: ConnectionState::Offline,
            peers: PeerStore::new(),
            local_peer_id,
            last_emitted_cursor: None,
            session,
            login_form: LoginForm::default(),
            server_http_base,
            refresh_rx: None,
            room_picker: RoomPicker::new(),
            current_room: std::env::var("BSE_ROOM").ok(),
            show_picker: false,
            last_broadcast_sv: Vec::new(),
            theme_applied: false,
        }
    }

    fn update_fps(&mut self) {
        let now = Instant::now();
        if let Some(prev) = self.last_frame {
            let dt = now.duration_since(prev).as_secs_f32();
            if dt > 0.0 {
                let instant = 1.0 / dt;
                self.fps = self.fps.mul_add(0.9, instant * 0.1);
            }
        }
        self.last_frame = Some(now);
    }

    fn rebuild_spatial(&mut self) {
        self.spatial.clear();
        for element in self.crdt.iter_elements() {
            self.spatial.insert(element.id, element.aabb());
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<egui::DroppedFile> = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            let result = if let Some(path) = file.path.as_ref() {
                self.assets.ingest_file(path)
            } else if let Some(bytes) = file.bytes {
                self.assets.ingest_bytes(bytes.to_vec())
            } else {
                continue;
            };
            match result {
                Ok((asset_id, w, h)) => {
                    if w > IMAGE_PIXEL_WARN || h > IMAGE_PIXEL_WARN {
                        warn!(target: "bse::assets", w, h, "large image accepted");
                    }
                    let element = canvas::commit_image(self.canvas.camera.position, asset_id, w, h);
                    if let Err(err) = self.crdt.upsert_element(element) {
                        warn!(target: "bse::app", error = %err, "image upsert failed");
                    }
                }
                Err(err) => {
                    warn!(target: "bse::assets", error = %err, "drop ignored");
                }
            }
        }
    }

    fn autosave_if_due(&mut self) {
        if !self.dirty {
            return;
        }
        if self.last_save.elapsed() < AUTOSAVE_INTERVAL {
            return;
        }
        let Some(storage) = self.storage.as_mut() else {
            return;
        };
        match self.crdt.encode_snapshot() {
            Ok(bytes) => match storage.save_snapshot(AUTOSAVE_KEY, &bytes) {
                Ok(()) => {
                    self.dirty = false;
                    self.last_save = Instant::now();
                    info!(
                        target: "bse::app",
                        elements = self.crdt.element_count(),
                        bytes = bytes.len(),
                        "autosaved",
                    );
                }
                Err(err) => warn!(target: "bse::app", error = %err, "save_snapshot failed"),
            },
            Err(err) => warn!(target: "bse::app", error = %err, "encode_snapshot failed"),
        }
    }

    fn force_save(&mut self) {
        self.dirty = true;
        self.last_save = Instant::now()
            .checked_sub(AUTOSAVE_INTERVAL)
            .unwrap_or_else(Instant::now);
        self.autosave_if_due();
    }

    fn handle_file_shortcuts(&mut self, ctx: &egui::Context) {
        let (save, open) = ctx.input(|i| {
            let cmd = i.modifiers.command;
            (
                cmd && i.key_pressed(egui::Key::S),
                cmd && i.key_pressed(egui::Key::O),
            )
        });
        if save {
            project_io::save_as_dialog(&self.crdt, "untitled");
        } else if open && project_io::open_dialog(&mut self.crdt).is_some() {
            self.canvas.tool_state = bse_canvas::ToolState::Idle;
            self.force_save();
        }
    }

    /// Pull events from the sync worker and update peer state.
    fn process_sync_events(&mut self) {
        let Some(sync) = self.sync.as_mut() else {
            return;
        };
        for event in sync.drain_events() {
            match event {
                SyncEvent::State(state) => {
                    self.connection_state = state;
                    if matches!(state, ConnectionState::Offline) {
                        self.peers.clear();
                    }
                }
                SyncEvent::PeerJoin {
                    peer_id,
                    display_name,
                    color,
                } => self.peers.on_join(peer_id, display_name, color),
                SyncEvent::PeerLeave(id) => self.peers.on_leave(id),
                SyncEvent::PeerCursor { peer_id, position } => {
                    if peer_id != self.local_peer_id {
                        self.peers.on_cursor(peer_id, position);
                    }
                }
                SyncEvent::RemoteOp(bytes) => {
                    if let Err(err) = self.crdt.apply_remote_update(&bytes) {
                        warn!(target: "bse::app", error = %err, "apply_remote_update failed");
                    } else {
                        self.dirty = true;
                        self.last_element_count = self.crdt.element_count();
                    }
                }
            }
        }
        self.peers.prune_stale();
    }

    /// Encode the current CRDT state delta and ship it to the room.
    ///
    /// v023 : ships *incremental* Yrs updates relative to
    /// [`Self::last_broadcast_sv`]. The first call after a fresh
    /// connection has an empty state vector and therefore sends a
    /// full snapshot — same payload as v010.3. Subsequent calls send
    /// only the bytes new peers do not have yet.
    ///
    /// Bandwidth gain on a 1000-element board : ~20× per mutation
    /// vs. shipping the whole document each time (measured locally).
    fn broadcast_local_state(&mut self) {
        let Some(sync) = self.sync.as_ref() else {
            return;
        };
        if !matches!(self.connection_state, ConnectionState::Connected) {
            return;
        }
        let bytes = match self.crdt.encode_update_since(&self.last_broadcast_sv) {
            Ok(b) => b,
            Err(err) => {
                warn!(target: "bse::app", error = %err, "encode_update_since failed");
                return;
            }
        };
        // Refresh our local checkpoint so the next call ships only
        // *new* mutations.
        match self.crdt.state_vector() {
            Ok(sv) => self.last_broadcast_sv = sv,
            Err(err) => warn!(target: "bse::app", error = %err, "state_vector failed"),
        }
        sync.send(SyncCmd::Op(bytes));
    }

    /// Kick off a background HTTP refresh if the access token is about
    /// to expire and no refresh is already in flight. Non-blocking : the
    /// new session lands on a `mpsc::Receiver` that is drained on the
    /// next frame via [`Self::poll_refresh`].
    fn maybe_refresh_token(&mut self) {
        if self.refresh_rx.is_some() || self.server_http_base.is_empty() {
            return;
        }
        let SessionState::SignedIn {
            access_expires_at,
            refresh_token,
            ..
        } = &self.session
        else {
            return;
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if now + REFRESH_LEAD_SECS < *access_expires_at {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let base = self.server_http_base.clone();
        let token = refresh_token.clone();
        std::thread::Builder::new()
            .name("bse-auth-refresh".into())
            .spawn(move || {
                let _ = tx.send(login::try_refresh(&base, &token));
            })
            .ok();
        self.refresh_rx = Some(rx);
        info!(target: "bse::auth", "kicking off background token refresh");
    }

    /// Drain the refresh channel (if any) and apply the result.
    fn poll_refresh(&mut self) {
        let Some(rx) = self.refresh_rx.as_ref() else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(res) => res,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.refresh_rx = None;
                return;
            }
        };
        self.refresh_rx = None;
        match result {
            Ok(new_session) => {
                self.session = new_session;
                if let Some(storage) = self.storage.as_mut() {
                    login::persist_session(storage, &self.session);
                }
                info!(target: "bse::auth", "session refreshed");
            }
            Err(msg) => {
                warn!(target: "bse::auth", error = %msg, "refresh failed ; signing out");
                self.sign_out();
            }
        }
    }

    /// Clear the local session, persist the wipe, and disconnect from
    /// the sync worker.
    fn sign_out(&mut self) {
        self.session = SessionState::SignedOut;
        if let Some(storage) = self.storage.as_mut() {
            login::clear_session(storage);
        }
        if let Some(sync) = self.sync.as_ref() {
            sync.send(SyncCmd::Disconnect);
        }
        self.peers.clear();
        self.room_picker.reset();
        // Keep `current_room` so that on re-login the user lands back
        // in the same room without having to pick again, but only if
        // it was originally provided via the env var.
        if std::env::var("BSE_ROOM").is_err() {
            self.current_room = None;
        }
        info!(target: "bse::auth", "signed out");
    }

    /// Make sure a sync worker exists and is connected to `self.current_room`.
    /// Called every frame once the user is signed in and a room is picked.
    fn ensure_sync_connected(&mut self) {
        if self.session.access_token().is_none() {
            return;
        }
        let Some(room_id) = self.current_room.clone() else {
            return;
        };
        if self.sync.is_none() {
            self.sync = Some(SyncHandle::spawn());
        }
        if let Some(handle) = self.sync.as_ref() {
            // The sync worker dedups duplicate Connect commands by
            // closing the previous client first, so resending on every
            // frame would be wasteful. We use the connection_state as
            // a coarse "have we connected at least once" signal.
            if matches!(self.connection_state, ConnectionState::Offline) {
                let token = self.session.access_token().map(str::to_owned);
                connect_handle(handle, &room_id, self.local_peer_id, token);
            }
        }
    }

    /// Switch to a new room : disconnect the current sync, wipe the
    /// local CRDT (it belongs to the previous room), and queue a fresh
    /// connect for the new room.
    fn switch_room(&mut self, new_room: String) {
        if self.current_room.as_deref() == Some(new_room.as_str()) {
            self.show_picker = false;
            return;
        }
        if let Some(handle) = self.sync.as_ref() {
            handle.send(SyncCmd::Disconnect);
        }
        self.current_room = Some(new_room);
        self.connection_state = ConnectionState::Offline;
        self.peers.clear();
        self.crdt = YrsBackend::new();
        self.last_element_count = 0;
        self.last_broadcast_sv.clear();
        self.show_picker = false;
        info!(target: "bse::app", room = ?self.current_room, "switched room");
    }

    /// Show the room picker if it has been opened manually or if the
    /// user just signed in and has not chosen a room yet.
    fn maybe_show_picker(&mut self, ctx: &egui::Context) {
        if !self.session.is_signed_in() || self.server_http_base.is_empty() {
            return;
        }
        let needs_pick = self.show_picker || self.current_room.is_none();
        if !needs_pick {
            return;
        }
        let Some(token) = self.session.access_token().map(str::to_owned) else {
            return;
        };
        if let Some(room) =
            self.room_picker
                .show(ctx, &self.server_http_base, &token)
        {
            self.switch_room(room);
        }
    }

    /// Render the small account widget on the right side of the toolbar
    /// (display name + sign-out button). No-op when signed out.
    fn render_account_widget(&mut self, ui: &mut egui::Ui) {
        let SessionState::SignedIn { display_name, .. } = &self.session else {
            return;
        };
        let display_name = display_name.clone();
        let room_label = self
            .current_room
            .clone()
            .unwrap_or_else(|| "(no room)".to_string());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("Sign out").clicked() {
                self.sign_out();
            }
            ui.add_space(8.0);
            if ui
                .small_button(format!("Room : {room_label}"))
                .on_hover_text("Click to switch room")
                .clicked()
            {
                self.show_picker = true;
            }
            ui.add_space(8.0);
            ui.label(format!("Signed in as {display_name}"));
        });
    }

    /// Show the login modal if the user is signed out AND a server is
    /// configured. Without a server URL there is nobody to authenticate
    /// against, so we silently stay offline.
    fn maybe_show_login(&mut self, ctx: &egui::Context) {
        if self.session.is_signed_in() || self.server_http_base.is_empty() {
            return;
        }
        if let Some(new_state) =
            login::show_modal(ctx, &mut self.login_form, &self.server_http_base)
        {
            self.session = new_state;
            if let Some(storage) = self.storage.as_mut() {
                login::persist_session(storage, &self.session);
            }
            if let SessionState::SignedIn { display_name, .. } = &self.session {
                info!(target: "bse::app", user = %display_name, "session established");
            }
        }
    }

    /// Read the local cursor screen position and forward it to the
    /// worker thread so it is broadcast as awareness.
    fn emit_local_cursor(&mut self, ctx: &egui::Context, canvas_rect: egui::Rect) {
        let Some(sync) = self.sync.as_ref() else {
            return;
        };
        if !matches!(self.connection_state, ConnectionState::Connected) {
            return;
        }
        let Some(pos_screen) = ctx.input(|i| i.pointer.hover_pos()) else {
            return;
        };
        if !canvas_rect.contains(pos_screen) {
            return;
        }
        let viewport = WorldVec2::new(canvas_rect.width(), canvas_rect.height());
        let local = pos_screen - canvas_rect.min.to_vec2();
        let screen = WorldVec2::new(local.x, local.y);
        let world = self.canvas.camera.screen_to_world(viewport, screen);
        if self.last_emitted_cursor != Some(world) {
            sync.send(SyncCmd::Cursor(world));
            self.last_emitted_cursor = Some(world);
        }
    }
}

/// Build a [`ClientConfig`] for `room_id` using env vars + the current
/// session for the auth token, then dispatch a [`SyncCmd::Connect`].
fn connect_handle(
    handle: &SyncHandle,
    room_id: &str,
    local_peer_id: PeerId,
    auth_token: Option<String>,
) -> bool {
    let Ok(url) = std::env::var("BSE_SERVER_URL") else {
        return false;
    };
    let display_name =
        std::env::var("BSE_DISPLAY_NAME").unwrap_or_else(|_| whoami_or_anonymous());
    handle.send(SyncCmd::Connect(ClientConfig {
        server_url: url,
        room_id: room_id.to_string(),
        peer_id: local_peer_id,
        display_name,
        auth_token,
    }));
    true
}

fn whoami_or_anonymous() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "anonymous".to_string())
}

fn open_default_storage() -> Option<SqliteStorage> {
    let dirs = directories::ProjectDirs::from("app", "BSE", "BSE")?;
    let data = dirs.data_local_dir();
    if let Err(err) = std::fs::create_dir_all(data) {
        warn!(target: "bse::storage", error = %err, "could not create data dir");
        return None;
    }
    let path = data.join("local.db");
    match SqliteStorage::open(&path) {
        Ok(s) => {
            info!(target: "bse::storage", path = %path.display(), "local storage ready");
            Some(s)
        }
        Err(err) => {
            warn!(target: "bse::storage", path = %path.display(), error = %err, "open failed");
            None
        }
    }
}

impl eframe::App for BseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            apply_bse_theme(ctx, ThemeMode::Light);
            self.theme_applied = true;
        }
        self.update_fps();
        self.process_sync_events();
        self.poll_refresh();
        self.maybe_refresh_token();
        self.maybe_show_login(ctx);
        self.maybe_show_picker(ctx);
        self.ensure_sync_connected();
        self.handle_file_shortcuts(ctx);
        self.handle_dropped_files(ctx);
        self.rebuild_spatial();

        // Top panel keeps only the account widget (right-aligned) so
        // the canvas runs full-height. The tool picker is drawn as a
        // floating bottom-centered bar inside the CentralPanel.
        egui::TopBottomPanel::top("account_bar")
            .exact_height(44.0)
            .show(ctx, |ui| {
                self.render_account_widget(ui);
            });

        let connection = match self.connection_state {
            ConnectionState::Offline => ConnectionStatus::Offline,
            ConnectionState::Connecting => ConnectionStatus::Connecting,
            ConnectionState::Connected => ConnectionStatus::Connected,
            ConnectionState::Reconnecting => ConnectionStatus::Reconnecting,
        };
        let peer_avatars: Vec<(String, egui::Color32)> = self
            .peers
            .iter()
            .map(|(_, p)| {
                let name = p
                    .display_name
                    .clone()
                    .unwrap_or_else(|| "Anon".to_string());
                let c = egui::Color32::from_rgb(p.color.r, p.color.g, p.color.b);
                (name, c)
            })
            .collect();
        let show_debug = std::env::var("BSE_DEBUG").is_ok();
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            status_bar(
                ui,
                &StatusInfo {
                    app: APP_INFO,
                    zoom: self.canvas.camera.zoom,
                    fps: self.fps,
                    peer_count: u32::try_from(self.peers.len()).unwrap_or(u32::MAX),
                    connection,
                    tool: self.canvas.tool,
                    element_count: u32::try_from(self.crdt.element_count()).unwrap_or(u32::MAX),
                    visible_count: self.last_visible_count,
                    peers: &peer_avatars,
                    show_debug,
                },
            );
        });

        let mut canvas_rect = egui::Rect::NOTHING;
        egui::CentralPanel::default().show(ctx, |ui| {
            self.last_visible_count = canvas::show(
                ui,
                &mut self.canvas,
                &mut self.crdt,
                &self.spatial,
                &mut self.assets,
                &self.peers,
            );
            // Floating tool picker centered-bottom on top of the canvas.
            toolbar(ui, &mut self.canvas);
            canvas_rect = ui.min_rect();
        });

        self.emit_local_cursor(ctx, canvas_rect);

        let current = self.crdt.element_count();
        if current != self.last_element_count {
            self.dirty = true;
            self.last_element_count = current;
            self.broadcast_local_state();
        }
        self.autosave_if_due();

        ctx.request_repaint();
    }

    fn on_exit(&mut self) {
        self.force_save();
    }
}
