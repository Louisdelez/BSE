//! Project metadata, persistence file format, and directory helpers for BSE.
//!
//! This crate is the **v015 foundation** : it owns the on-disk shape of a
//! BSE project but does not know what a "scene" actually contains. The
//! scene snapshot is treated as an opaque `&[u8]` blob produced and
//! consumed by `bse-crdt`.
//!
//! # The `.bse` file format
//!
//! A `.bse` file is a standard `ZIP` archive (deflate-compressed) with
//! the following layout :
//!
//! ```text
//! my-project.bse
//! ├── project.json   # required, UTF-8 JSON, pretty-printed
//! ├── scene.bin      # required, opaque snapshot bytes
//! └── assets/        # optional, may be absent for v015
//!     ├── <sha256>   # content-addressed asset blob
//!     └── ...
//! ```
//!
//! Unknown entries are tolerated on load. Saving never writes anything
//! outside of `project.json` and `scene.bin` ; asset support is reserved
//! for a later version of the format.
//!
//! ## `project.json` schema
//!
//! ```json
//! {
//!   "id":             "<uuid-v7>",
//!   "name":           "My Board",
//!   "created_at":     1_700_000_000,
//!   "updated_at":     1_700_000_000,
//!   "created_by":     "<uuid-v7>" | null,
//!   "schema_version": 1
//! }
//! ```
//!
//! Timestamps are seconds since the Unix epoch. `created_by` is `null`
//! for projects authored offline. The current schema version is
//! [`metadata::CURRENT_SCHEMA_VERSION`] ; loading a file with a newer
//! version yields [`ProjectError::UnsupportedSchema`].
//!
//! # Save & load
//!
//! ```no_run
//! use std::path::Path;
//! use bse_projects::{ProjectMetadata, save_to_file, load_from_file};
//!
//! let meta = ProjectMetadata::new("Demo");
//! let scene: Vec<u8> = vec![]; // from bse-crdt
//! save_to_file(Path::new("demo.bse"), &meta, &scene).unwrap();
//!
//! let loaded = load_from_file(Path::new("demo.bse")).unwrap();
//! assert_eq!(loaded.metadata.name, "Demo");
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod dir;
pub mod error;
pub mod file;
pub mod metadata;

pub use dir::{PROJECT_EXTENSION, default_project_dir, list_projects_in_dir};
pub use error::ProjectError;
pub use file::{LoadedProject, load_from_file, save_to_file};
pub use metadata::{CURRENT_SCHEMA_VERSION, ProjectMetadata};
