//! Thin binary entry point. All app logic lives in the library.
//!
//! Keeping `main` tiny lets integration tests reuse `bse_app::run()`.

fn main() -> Result<(), eframe::Error> {
    bse_app::run()
}
