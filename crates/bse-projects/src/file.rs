//! Save / load a project to a single `.bse` zip archive.
//!
//! The archive layout is documented at the crate root (see [`crate`]).
//! This module knows only about three logical entries :
//!
//! - `project.json` (required) — pretty-printed [`ProjectMetadata`].
//! - `scene.bin`    (required) — opaque snapshot bytes from the CRDT layer.
//! - `assets/*`     (optional) — content-addressed asset blobs (not read here).
//!
//! Save is atomic in the usual filesystem sense : we write to a sibling
//! `*.tmp` file and rename it into place once the zip writer has flushed.
//! Load tolerates extra entries (e.g. assets) without complaint.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tracing::{debug, trace};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::ProjectError;
use crate::metadata::{CURRENT_SCHEMA_VERSION, ProjectMetadata};

/// Name of the JSON metadata entry inside the archive.
pub(crate) const PROJECT_JSON: &str = "project.json";

/// Name of the binary scene-snapshot entry inside the archive.
pub(crate) const SCENE_BIN: &str = "scene.bin";

/// A project loaded from disk : metadata + the raw scene blob.
///
/// The scene bytes are returned uninterpreted — it is up to the caller
/// (typically `bse-crdt`) to deserialise them.
#[derive(Clone, Debug)]
pub struct LoadedProject {
    /// Parsed `project.json` content.
    pub metadata: ProjectMetadata,
    /// Raw contents of the `scene.bin` entry.
    pub scene_bytes: Vec<u8>,
}

/// Persist a project to `path` as a `.bse` zip archive.
///
/// Writes go through a sibling `<path>.tmp` file that is renamed into
/// place on success ; partially-written archives never overwrite a good
/// one. Any pre-existing `<path>.tmp` is silently overwritten.
///
/// The parent directory is created if missing.
pub fn save_to_file(
    path: &Path,
    metadata: &ProjectMetadata,
    scene_bytes: &[u8],
) -> Result<(), ProjectError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
        debug!(parent = %parent.display(), "created project parent directory");
    }

    let tmp = tmp_path(path);
    // Scope the writer so its file handle is dropped before the rename.
    {
        let file = File::create(&tmp)?;
        let mut zip = ZipWriter::new(file);

        let json = serde_json::to_vec_pretty(metadata)?;
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        zip.start_file(PROJECT_JSON, opts)?;
        zip.write_all(&json)?;

        zip.start_file(SCENE_BIN, opts)?;
        zip.write_all(scene_bytes)?;

        zip.finish()?;
    }

    fs::rename(&tmp, path)?;
    trace!(path = %path.display(), bytes = scene_bytes.len(), "saved project");
    Ok(())
}

/// Load a `.bse` archive from `path`.
///
/// Returns [`ProjectError::MissingEntry`] if either `project.json` or
/// `scene.bin` is absent, and [`ProjectError::UnsupportedSchema`] if the
/// metadata advertises a `schema_version` newer than this build.
pub fn load_from_file(path: &Path) -> Result<LoadedProject, ProjectError> {
    let file = File::open(path)?;
    let mut zip = ZipArchive::new(file)?;

    let metadata = read_metadata(&mut zip)?;
    if metadata.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(ProjectError::UnsupportedSchema(metadata.schema_version));
    }

    let scene_bytes = read_entry(&mut zip, SCENE_BIN)?;

    trace!(
        path = %path.display(),
        bytes = scene_bytes.len(),
        schema = metadata.schema_version,
        "loaded project"
    );
    Ok(LoadedProject {
        metadata,
        scene_bytes,
    })
}

/// Read and parse `project.json` from an opened archive.
///
/// Extracted so [`crate::dir::list_projects_in_dir`] can reuse it without
/// also paying for `scene.bin`.
pub(crate) fn read_metadata<R: Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
) -> Result<ProjectMetadata, ProjectError> {
    let bytes = read_entry(zip, PROJECT_JSON)?;
    let metadata: ProjectMetadata = serde_json::from_slice(&bytes)?;
    Ok(metadata)
}

/// Read a single named entry into a `Vec<u8>`. Mapped to
/// [`ProjectError::MissingEntry`] when absent.
fn read_entry<R: Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
    name: &str,
) -> Result<Vec<u8>, ProjectError> {
    let mut entry = match zip.by_name(name) {
        Ok(e) => e,
        Err(zip::result::ZipError::FileNotFound) => {
            return Err(ProjectError::MissingEntry(name.to_owned()));
        }
        Err(other) => return Err(other.into()),
    };
    let mut buf = Vec::with_capacity(usize::try_from(entry.size()).unwrap_or(0));
    entry.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Compute the sibling temp path used by [`save_to_file`].
fn tmp_path(path: &Path) -> PathBuf {
    let mut p = path.as_os_str().to_owned();
    p.push(".tmp");
    PathBuf::from(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn save_load_roundtrip_preserves_data() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("hello.bse");

        let meta = ProjectMetadata::new("Hello");
        let scene = b"the quick brown fox jumps over the lazy dog".to_vec();

        save_to_file(&path, &meta, &scene).expect("save");
        assert!(path.exists());
        // No stray .tmp.
        assert!(!tmp_path(&path).exists());

        let loaded = load_from_file(&path).expect("load");
        assert_eq!(loaded.metadata, meta);
        assert_eq!(loaded.scene_bytes, scene);
    }

    #[test]
    fn load_missing_file_returns_io_error() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("nope.bse");

        let err = load_from_file(&path).expect_err("missing file");
        assert!(matches!(err, ProjectError::Io(_)), "got : {err:?}");
    }

    #[test]
    fn load_non_zip_returns_error() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("not-a-zip.bse");
        fs::write(&path, b"definitely not a zip archive").unwrap();

        let err = load_from_file(&path).expect_err("non-zip");
        // The `zip` crate may surface this as `Io` or `Zip` depending on
        // how it fails to find the central directory ; accept either.
        assert!(
            matches!(err, ProjectError::Zip(_) | ProjectError::Io(_)),
            "got : {err:?}"
        );
    }

    #[test]
    fn load_zip_without_project_json_returns_missing_entry() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("empty.bse");
        {
            let f = File::create(&path).unwrap();
            let mut z = ZipWriter::new(f);
            z.start_file("other.txt", SimpleFileOptions::default())
                .unwrap();
            z.write_all(b"hi").unwrap();
            z.finish().unwrap();
        }

        let err = load_from_file(&path).expect_err("no project.json");
        match err {
            ProjectError::MissingEntry(name) => assert_eq!(name, PROJECT_JSON),
            other => panic!("expected MissingEntry, got {other:?}"),
        }
    }

    #[test]
    fn load_zip_without_scene_returns_missing_entry() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("no-scene.bse");
        {
            let f = File::create(&path).unwrap();
            let mut z = ZipWriter::new(f);
            z.start_file(PROJECT_JSON, SimpleFileOptions::default())
                .unwrap();
            let meta = ProjectMetadata::new("X");
            z.write_all(&serde_json::to_vec_pretty(&meta).unwrap())
                .unwrap();
            z.finish().unwrap();
        }

        let err = load_from_file(&path).expect_err("no scene.bin");
        match err {
            ProjectError::MissingEntry(name) => assert_eq!(name, SCENE_BIN),
            other => panic!("expected MissingEntry, got {other:?}"),
        }
    }

    #[test]
    fn unsupported_schema_version_is_reported() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("future.bse");
        {
            let f = File::create(&path).unwrap();
            let mut z = ZipWriter::new(f);
            let mut meta = ProjectMetadata::new("Future");
            meta.schema_version = CURRENT_SCHEMA_VERSION + 42;
            z.start_file(PROJECT_JSON, SimpleFileOptions::default())
                .unwrap();
            z.write_all(&serde_json::to_vec_pretty(&meta).unwrap())
                .unwrap();
            z.start_file(SCENE_BIN, SimpleFileOptions::default())
                .unwrap();
            z.write_all(b"").unwrap();
            z.finish().unwrap();
        }

        let err = load_from_file(&path).expect_err("future schema");
        match err {
            ProjectError::UnsupportedSchema(v) => {
                assert_eq!(v, CURRENT_SCHEMA_VERSION + 42);
            }
            other => panic!("expected UnsupportedSchema, got {other:?}"),
        }
    }

    #[test]
    fn save_creates_missing_parent_dirs() {
        let dir = tempdir().expect("tempdir");
        let nested = dir.path().join("a").join("b").join("c").join("x.bse");
        assert!(!nested.parent().unwrap().exists());

        let meta = ProjectMetadata::new("Nested");
        save_to_file(&nested, &meta, b"hi").expect("save");
        assert!(nested.exists());
    }
}
