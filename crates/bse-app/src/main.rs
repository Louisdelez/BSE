//! BSE desktop app entry point.
//!
//! In v002 the binary is a simple smoke test that proves the workspace
//! links and runs : it prints app metadata and exits successfully.
//! v003 swaps `main` for an `eframe::run_native` call.

use bse_canvas::CanvasState;
use bse_crdt::InMemoryBackend;
use bse_render::{NullRenderer, Renderer};
use bse_types::Vec2;
use bse_ui::AppInfo;
use tracing::info;

const APP_INFO: AppInfo = AppInfo {
    name: "BSE",
    version: env!("CARGO_PKG_VERSION"),
    milestone: "v002",
};

fn main() {
    init_tracing();
    let title = APP_INFO.title();
    info!("starting {title}");

    // v002 smoke test : exercise every crate so the workspace
    // builds and links end-to-end.
    let canvas = CanvasState::new();
    let backend = InMemoryBackend::new();
    let mut renderer = NullRenderer::new();
    let stats = renderer.render(backend.scene(), &canvas.camera, Vec2::new(1920.0, 1080.0));

    info!(
        target: "bse::app",
        elements_total = stats.elements_total,
        elements_visible = stats.elements_visible,
        "smoke test passed",
    );

    let visible = stats.elements_visible;
    println!("✓ {title} ready (smoke test : {visible} elements rendered)");
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
