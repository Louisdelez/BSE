//! Save / load the current document as a `.bse` file.
//!
//! Native file dialogs come from the `rfd` crate. On Linux they are
//! the desktop's XDG portal. On Windows and macOS they are the native
//! OS dialogs.

use std::path::PathBuf;

use bse_crdt::{CrdtBackend, YrsBackend};
use bse_projects::{LoadedProject, ProjectMetadata, load_from_file, save_to_file};
use tracing::{info, warn};

/// File extension expected for project files.
const EXT: &str = "bse";

/// Show a native "save as" dialog and write the current document
/// to the chosen path.
///
/// Returns the destination path on success, or `None` if the user
/// cancelled the dialog.
pub fn save_as_dialog(crdt: &YrsBackend, project_name: &str) -> Option<PathBuf> {
    let default_name = format!("{project_name}.{EXT}");
    let starting_dir = bse_projects::default_project_dir().ok();
    let mut builder = rfd::FileDialog::new()
        .add_filter("BSE project", &[EXT])
        .set_file_name(default_name);
    if let Some(dir) = starting_dir {
        builder = builder.set_directory(dir);
    }
    let path = builder.save_file()?;
    let metadata = ProjectMetadata::new(project_name);
    match crdt.encode_snapshot() {
        Ok(bytes) => match save_to_file(&path, &metadata, &bytes) {
            Ok(()) => {
                info!(
                    target: "bse::project",
                    path = %path.display(),
                    elements = crdt.element_count(),
                    "saved",
                );
                Some(path)
            }
            Err(err) => {
                warn!(target: "bse::project", error = %err, "save failed");
                None
            }
        },
        Err(err) => {
            warn!(target: "bse::project", error = %err, "encode_snapshot failed");
            None
        }
    }
}

/// Show a native "open" dialog and load the chosen project into the
/// CRDT document, replacing its current content.
///
/// Returns the loaded project metadata on success, or `None` if the
/// user cancelled or the file couldn't be parsed.
pub fn open_dialog(crdt: &mut YrsBackend) -> Option<ProjectMetadata> {
    let starting_dir = bse_projects::default_project_dir().ok();
    let mut builder = rfd::FileDialog::new().add_filter("BSE project", &[EXT]);
    if let Some(dir) = starting_dir {
        builder = builder.set_directory(dir);
    }
    let path = builder.pick_file()?;
    match load_from_file(&path) {
        Ok(LoadedProject {
            metadata,
            scene_bytes,
        }) => {
            // Replace the current doc with a fresh one, then apply the
            // loaded snapshot. Bypasses CRDT merging on purpose : the
            // user explicitly asked to open a different project.
            *crdt = YrsBackend::new();
            if let Err(err) = crdt.apply_remote_update(&scene_bytes) {
                warn!(target: "bse::project", error = %err, "load: apply_remote_update failed");
                return None;
            }
            info!(
                target: "bse::project",
                path = %path.display(),
                name = %metadata.name,
                elements = crdt.element_count(),
                "loaded",
            );
            Some(metadata)
        }
        Err(err) => {
            warn!(target: "bse::project", error = %err, "load_from_file failed");
            None
        }
    }
}
