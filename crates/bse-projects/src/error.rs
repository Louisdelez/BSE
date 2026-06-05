//! Errors produced by the `bse-projects` crate.

/// Errors that can occur when reading, writing, or listing `.bse` files.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// An underlying I/O error (file open, read, write, directory ops).
    ///
    /// The original [`std::io::Error`] is stringified to keep the error
    /// type `Clone`-free of platform quirks and trivially `Send + Sync`.
    #[error("I/O : {0}")]
    Io(String),

    /// An error from the `zip` crate (corrupt archive, unsupported method,
    /// invalid central directory, ...).
    #[error("zip : {0}")]
    Zip(String),

    /// The file opened correctly as a zip but its content does not match
    /// the `.bse` schema (e.g. `project.json` is not valid JSON, or a
    /// timestamp is missing).
    #[error("malformed project file : {0}")]
    Malformed(String),

    /// A required entry was absent from the archive (e.g. no `project.json`
    /// or no `scene.bin`).
    #[error("missing entry : {0}")]
    MissingEntry(String),

    /// The `schema_version` field in `project.json` is newer than this
    /// build knows how to read.
    #[error("unsupported schema version : {0}")]
    UnsupportedSchema(u32),
}

impl From<std::io::Error> for ProjectError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<zip::result::ZipError> for ProjectError {
    fn from(value: zip::result::ZipError) -> Self {
        // `ZipError::Io` is the most common variant from misuse ; surface
        // it as `Io` to give callers a less zip-specific error to match on.
        match value {
            zip::result::ZipError::Io(e) => Self::Io(e.to_string()),
            other => Self::Zip(other.to_string()),
        }
    }
}

impl From<serde_json::Error> for ProjectError {
    fn from(value: serde_json::Error) -> Self {
        Self::Malformed(value.to_string())
    }
}
