//! BSE desktop application.
//!
//! The app is built on top of `eframe` (which composes `egui` + `wgpu` +
//! `winit`). The library exposes a single [`run`] function that starts
//! the event loop ; the binary just calls it.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod app;
mod assets;
mod canvas;
mod project_io;
mod tracing_setup;

pub use assets::{Asset, AssetError, AssetStore};

use bse_ui::AppInfo;
use eframe::{NativeOptions, egui};
use tracing::info;

use crate::app::BseApp;

/// Static metadata for the running binary.
pub const APP_INFO: AppInfo = AppInfo {
    name: "BSE",
    version: env!("CARGO_PKG_VERSION"),
    milestone: "v015.1",
};

/// Start the event loop and run BSE until the user closes the window.
///
/// # Errors
///
/// Returns whatever error `eframe` reports if window creation fails.
pub fn run() -> Result<(), eframe::Error> {
    tracing_setup::init();
    let title = APP_INFO.title();
    info!("starting {title}");

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title(title),
        ..Default::default()
    };

    eframe::run_native("BSE", options, Box::new(|_cc| Ok(Box::new(BseApp::new()))))
}
