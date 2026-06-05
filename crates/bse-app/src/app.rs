//! Top-level [`eframe::App`] implementation for BSE.
//!
//! Owns the canvas state, the CRDT-backed document, the spatial index,
//! the asset store, and the local persistence handle. Coordinates the
//! layout between toolbar, canvas, and status bar.

use std::time::{Duration, Instant};

use bse_canvas::CanvasState;
use bse_crdt::{CrdtBackend, YrsBackend};
use bse_spatial::Quadtree;
use bse_storage::{LocalStorage, SqliteStorage};
use bse_types::{ElementId, Rect as WorldRect, Vec2 as WorldVec2};
use bse_ui::{StatusInfo, status_bar, toolbar};
use eframe::egui;
use tracing::{info, warn};

use crate::APP_INFO;
use crate::assets::AssetStore;
use crate::canvas;
use crate::project_io;

const SPATIAL_HALF_EXTENT: f32 = 1_000_000.0;
const SPATIAL_MAX_ITEMS_PER_LEAF: usize = 16;
const SPATIAL_MAX_DEPTH: u32 = 10;

/// Key used to autosave the current document in `SqliteStorage`.
const AUTOSAVE_KEY: &str = "current-project";
/// Minimum interval between two autosaves.
const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(5);

const IMAGE_PIXEL_WARN: u32 = 4096;

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
}

impl Default for BseApp {
    fn default() -> Self {
        Self::new()
    }
}

impl BseApp {
    /// Build a fresh app. Loads the previous session from the local
    /// `SQLite` store if present ; falls back to an empty document.
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
        // Stale-out `last_save` so the autosave guard always fires here.
        self.last_save = Instant::now()
            .checked_sub(AUTOSAVE_INTERVAL)
            .unwrap_or_else(Instant::now);
        self.autosave_if_due();
    }

    /// React to Ctrl+S (save as) and Ctrl+O (open) keyboard shortcuts.
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
            // Reset transient UI state and force one autosave so the
            // local cache reflects the freshly-loaded document.
            self.canvas.tool_state = bse_canvas::ToolState::Idle;
            self.force_save();
        }
    }
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
        self.update_fps();
        self.handle_file_shortcuts(ctx);
        self.handle_dropped_files(ctx);
        self.rebuild_spatial();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar(ui, &mut self.canvas);
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            status_bar(
                ui,
                StatusInfo {
                    app: APP_INFO,
                    zoom: self.canvas.camera.zoom,
                    fps: self.fps,
                    peer_count: 0,
                    tool: self.canvas.tool,
                    element_count: u32::try_from(self.crdt.element_count()).unwrap_or(u32::MAX),
                    visible_count: self.last_visible_count,
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.last_visible_count = canvas::show(
                ui,
                &mut self.canvas,
                &mut self.crdt,
                &self.spatial,
                &mut self.assets,
            );
        });

        let current = self.crdt.element_count();
        if current != self.last_element_count {
            self.dirty = true;
            self.last_element_count = current;
        }
        self.autosave_if_due();

        ctx.request_repaint();
    }

    fn on_exit(&mut self) {
        self.force_save();
    }
}
