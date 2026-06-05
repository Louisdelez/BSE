//! Directory-level helpers : list `.bse` files and locate the OS-conventional
//! storage directory for BSE projects.

use std::fs;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use tracing::warn;
use zip::ZipArchive;

use crate::error::ProjectError;
use crate::file::read_metadata;
use crate::metadata::ProjectMetadata;

/// Conventional file extension for BSE projects.
pub const PROJECT_EXTENSION: &str = "bse";

/// List the metadata of every `.bse` file in `dir`.
///
/// Subdirectories are not recursed. Individual files that fail to open
/// or parse are skipped with a `tracing::warn!` — the function only
/// returns `Err` for failures of the directory iteration itself (e.g.
/// `dir` does not exist, permission denied, ...).
///
/// Order of the returned vector mirrors the OS iteration order ; do
/// not rely on it being sorted.
pub fn list_projects_in_dir(dir: &Path) -> Result<Vec<ProjectMetadata>, ProjectError> {
    let mut out = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, dir = %dir.display(), "failed to read directory entry");
                continue;
            }
        };

        let path = entry.path();
        if !is_bse_file(&path) {
            continue;
        }

        match read_metadata_only(&path) {
            Ok(meta) => out.push(meta),
            Err(e) => {
                warn!(
                    error = %e,
                    path = %path.display(),
                    "skipping invalid .bse file"
                );
            }
        }
    }

    Ok(out)
}

/// Return the OS-conventional directory where BSE stores user projects,
/// creating it if missing.
///
/// Paths used :
///
/// | Platform | Path                                                 |
/// |----------|------------------------------------------------------|
/// | Windows  | `%APPDATA%\BSE\projects\`                            |
/// | macOS    | `~/Library/Application Support/BSE/projects/`        |
/// | Linux    | `~/.local/share/bse/projects/` (`$XDG_DATA_HOME`)    |
///
/// Returns [`ProjectError::Io`] if the platform has no recognised home
/// directory (extremely rare ; happens in sandbox environments without
/// `$HOME`) or if the directory cannot be created.
pub fn default_project_dir() -> Result<PathBuf, ProjectError> {
    let dir = project_data_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// Build the OS-conventional project directory path without creating it.
///
/// The `directories` crate's `ProjectDirs::data_dir()` inserts an extra
/// `data` subdirectory on Windows (giving `%APPDATA%\BSE\data\projects`),
/// which does not match the path we promise in the public docs. We
/// therefore reach down to [`BaseDirs`] and build the BSE-rooted path
/// ourselves, matching the table in [`default_project_dir`].
///
/// Split out so tests can inspect the path the platform actually returns.
fn project_data_dir() -> Result<PathBuf, ProjectError> {
    let base = BaseDirs::new()
        .ok_or_else(|| ProjectError::Io("no home directory available".to_owned()))?;

    // `data_dir()` returns the platform-appropriate root :
    //   Windows : `%APPDATA%`             (= `…\AppData\Roaming`)
    //   macOS   : `~/Library/Application Support`
    //   Linux   : `$XDG_DATA_HOME` or `~/.local/share`
    let root = base.data_dir();

    // Match the casing convention of each platform : `BSE` on Windows
    // and macOS (mixed-case app folders are the norm), lowercase `bse`
    // on Linux (lowercase XDG paths are the norm).
    #[cfg(target_os = "linux")]
    let app = "bse";
    #[cfg(not(target_os = "linux"))]
    let app = "BSE";

    Ok(root.join(app).join("projects"))
}

/// `true` if `path` is a regular file with a `.bse` extension.
fn is_bse_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case(PROJECT_EXTENSION))
}

/// Open `path` as a zip archive and parse `project.json` only.
///
/// Equivalent to [`crate::load_from_file`] but skips the `scene.bin`
/// read — useful for project pickers that want to display dozens of
/// projects without pulling every snapshot into memory.
fn read_metadata_only(path: &Path) -> Result<ProjectMetadata, ProjectError> {
    let file = fs::File::open(path)?;
    let mut zip = ZipArchive::new(file)?;
    read_metadata(&mut zip)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::save_to_file;
    use tempfile::tempdir;

    #[test]
    fn list_returns_valid_skips_invalid() {
        let dir = tempdir().expect("tempdir");

        // Two valid projects.
        let m1 = ProjectMetadata::new("Alpha");
        let m2 = ProjectMetadata::new("Beta");
        save_to_file(&dir.path().join("alpha.bse"), &m1, b"a").unwrap();
        save_to_file(&dir.path().join("beta.bse"), &m2, b"b").unwrap();

        // One file with the right extension but garbage content.
        fs::write(dir.path().join("broken.bse"), b"not a zip").unwrap();

        // A non-.bse file that must be ignored.
        fs::write(dir.path().join("notes.txt"), b"hi").unwrap();

        // A nested directory that must be ignored (no recursion).
        fs::create_dir(dir.path().join("sub")).unwrap();
        save_to_file(&dir.path().join("sub").join("ignored.bse"), &m1, b"c").unwrap();

        let mut got = list_projects_in_dir(dir.path()).expect("list");
        got.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(got.len(), 2, "expected 2 valid projects, got {got:?}");
        assert_eq!(got[0].name, "Alpha");
        assert_eq!(got[1].name, "Beta");
    }

    #[test]
    fn list_empty_dir_returns_empty_vec() {
        let dir = tempdir().expect("tempdir");
        let got = list_projects_in_dir(dir.path()).expect("list");
        assert!(got.is_empty());
    }

    #[test]
    fn list_missing_dir_returns_err() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist");
        let err = list_projects_in_dir(&missing).expect_err("missing dir");
        assert!(matches!(err, ProjectError::Io(_)), "got : {err:?}");
    }

    #[test]
    fn default_project_dir_creates_and_returns_path() {
        // We don't want to spam the real APPDATA in CI ; just check that
        // the function returns a path and the directory exists afterwards.
        // ProjectDirs is deterministic per-user, so subsequent runs
        // hit an existing directory — both branches are covered across
        // a fresh checkout + a second invocation.
        let path = default_project_dir().expect("default dir");
        assert!(path.is_absolute(), "path must be absolute : {path:?}");
        assert!(path.ends_with("projects"), "path tail : {path:?}");
        assert!(path.exists(), "directory must exist after call : {path:?}");
        assert!(path.is_dir(), "must be a directory : {path:?}");
    }
}
