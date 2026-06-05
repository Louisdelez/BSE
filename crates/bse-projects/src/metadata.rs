//! Project-level metadata stored alongside the scene blob.

use std::time::{SystemTime, UNIX_EPOCH};

use bse_types::{ProjectId, UserId};
use serde::{Deserialize, Serialize};

/// The current `.bse` schema version. Bump on any incompatible change to
/// the archive layout or the [`ProjectMetadata`] JSON shape.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Metadata describing a single BSE project.
///
/// Stored as `project.json` inside the `.bse` zip archive. Kept small and
/// JSON-friendly so external tooling can read it without a Rust dependency.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Stable identifier for this project. Generated as a UUID v7 by
    /// [`ProjectMetadata::new`] so newer projects sort after older ones.
    pub id: ProjectId,

    /// Human-readable project name. Free-form text ; not used for the
    /// on-disk filename.
    pub name: String,

    /// Creation time, in seconds since the Unix epoch.
    pub created_at: u64,

    /// Last-modified time, in seconds since the Unix epoch. Bumped by
    /// [`ProjectMetadata::touch`].
    pub updated_at: u64,

    /// Optional creator user id. May be `None` for projects authored in
    /// offline / single-user mode where no account exists yet.
    pub created_by: Option<UserId>,

    /// Schema version of the on-disk format. See [`CURRENT_SCHEMA_VERSION`].
    pub schema_version: u32,
}

impl ProjectMetadata {
    /// Create a fresh metadata with sensible defaults.
    ///
    /// - `id`             = new UUID v7 (time-ordered).
    /// - `created_at`     = `updated_at` = current Unix time in seconds.
    /// - `created_by`     = `None`.
    /// - `schema_version` = [`CURRENT_SCHEMA_VERSION`].
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let now = unix_now_secs();
        Self {
            id: ProjectId::new_v7(),
            name: name.into(),
            created_at: now,
            updated_at: now,
            created_by: None,
            schema_version: CURRENT_SCHEMA_VERSION,
        }
    }

    /// Set [`Self::updated_at`] to the current Unix time in seconds.
    ///
    /// Idempotent if called twice within the same second.
    pub fn touch(&mut self) {
        self.updated_at = unix_now_secs();
    }
}

/// Current Unix timestamp in seconds, saturating at `0` if the clock is
/// somehow set before the epoch.
fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_defaults() {
        let m = ProjectMetadata::new("My Board");
        assert_eq!(m.name, "My Board");
        assert_eq!(m.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(m.created_by.is_none());
        assert_eq!(m.created_at, m.updated_at);
        // `now` should be well past 2020-01-01.
        assert!(m.created_at > 1_577_836_800);
    }

    #[test]
    fn touch_updates_timestamp() {
        let mut m = ProjectMetadata::new("X");
        // Roll back so touch() is guaranteed to advance the value, even if
        // the test runs entirely within a single OS-clock second.
        m.updated_at = 0;
        m.touch();
        assert!(m.updated_at > 0);
    }

    #[test]
    fn json_roundtrip() {
        let m = ProjectMetadata::new("Sample");
        let s = serde_json::to_string_pretty(&m).expect("serialize");
        let back: ProjectMetadata = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(m, back);
    }
}
