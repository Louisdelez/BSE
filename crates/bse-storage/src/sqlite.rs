//! `SQLite`-backed implementation of [`LocalStorage`].
//!
//! Persists snapshots in a single `snapshots(key, bytes, updated_at)` table.
//! Uses WAL journal mode for crash-safety and reader/writer concurrency.
//! The `rusqlite` `bundled` feature is enabled at the crate level so no
//! system `SQLite` library is required on the build host.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use tracing::{debug, trace};

use crate::{LocalStorage, StorageError};

/// DDL applied at every open; idempotent thanks to `IF NOT EXISTS`.
const SCHEMA_SQL: &str = "CREATE TABLE IF NOT EXISTS snapshots (\
        key        TEXT    PRIMARY KEY, \
        bytes      BLOB    NOT NULL, \
        updated_at INTEGER NOT NULL\
    )";

/// Upsert SQL used by [`LocalStorage::save_snapshot`].
const UPSERT_SQL: &str = "INSERT INTO snapshots (key, bytes, updated_at) \
     VALUES (?1, ?2, ?3) \
     ON CONFLICT(key) DO UPDATE SET \
        bytes = excluded.bytes, \
        updated_at = excluded.updated_at";

/// Select SQL used by [`LocalStorage::load_snapshot`].
const SELECT_SQL: &str = "SELECT bytes FROM snapshots WHERE key = ?1";

/// `SQLite`-backed storage for project snapshots.
///
/// Construct via [`SqliteStorage::open`] for a file-based database or
/// [`SqliteStorage::in_memory`] for an ephemeral one (useful in tests).
///
/// The underlying [`rusqlite::Connection`] is `Send` but not `Sync` (it
/// caches prepared statements in a `RefCell`), so we wrap it in a
/// [`Mutex`] to satisfy the [`LocalStorage`] trait's `Send + Sync` bound.
/// Local storage is not on any hot path, so the lock contention cost is
/// negligible.
pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    /// Open or create a `SQLite`-backed storage at `path`.
    ///
    /// Creates the parent directory if missing, ensures the `snapshots`
    /// table exists, and switches the journal to WAL mode for
    /// crash-safety and concurrent readers.
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
            debug!(parent = %parent.display(), "created storage parent directory");
        }

        let conn = Connection::open(path)?;
        debug!(path = %path.display(), "opened sqlite storage");
        Self::init_file(conn)
    }

    /// Open an in-memory database, useful for tests.
    ///
    /// The schema is created but WAL is not applied (in-memory databases
    /// do not support WAL ; `journal_mode` stays at the `SQLite` default).
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Apply file-only pragmas (WAL etc.) and create the schema.
    fn init_file(conn: Connection) -> Result<Self, StorageError> {
        // WAL improves crash-safety and lets readers proceed during writes.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Lock the connection mutex, mapping poison errors to [`StorageError`].
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StorageError> {
        self.conn
            .lock()
            .map_err(|_| StorageError::Database("connection mutex poisoned".into()))
    }
}

impl LocalStorage for SqliteStorage {
    fn save_snapshot(&mut self, key: &str, bytes: &[u8]) -> Result<(), StorageError> {
        let updated_at = unix_now_secs();
        let conn = self.lock()?;
        let n = conn.execute(UPSERT_SQL, params![key, bytes, updated_at])?;
        trace!(key, bytes = bytes.len(), rows = n, "saved snapshot");
        Ok(())
    }

    fn load_snapshot(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let conn = self.lock()?;
        let row = conn
            .query_row(SELECT_SQL, params![key], |row| row.get::<_, Vec<u8>>(0))
            .optional()?;
        trace!(key, hit = row.is_some(), "loaded snapshot");
        Ok(row)
    }
}

/// Current Unix timestamp in seconds, saturating at `0` if the system
/// clock is somehow before the epoch.
fn unix_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Build a unique temp directory and return a path inside it.
    ///
    /// We avoid the `tempfile` crate to keep the dev-dep surface zero ;
    /// the path is namespaced by pid + nanos so parallel tests never
    /// collide.
    fn temp_path(label: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("bse-storage-{pid}-{nanos}-{label}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir.join("snapshots.sqlite")
    }

    /// Count rows in the `snapshots` table (test helper).
    fn row_count(s: &SqliteStorage) -> i64 {
        let conn = s.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn save_and_load_roundtrip() {
        let mut s = SqliteStorage::in_memory().unwrap();
        s.save_snapshot("proj-1", b"hello world").unwrap();
        let got = s.load_snapshot("proj-1").unwrap();
        assert_eq!(got.as_deref(), Some(&b"hello world"[..]));
    }

    #[test]
    fn load_missing_key_returns_none() {
        let s = SqliteStorage::in_memory().unwrap();
        let got = s.load_snapshot("does-not-exist").unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn save_twice_overwrites_single_row() {
        let mut s = SqliteStorage::in_memory().unwrap();
        s.save_snapshot("k", b"first").unwrap();
        s.save_snapshot("k", b"second").unwrap();

        let got = s.load_snapshot("k").unwrap();
        assert_eq!(got.as_deref(), Some(&b"second"[..]));
        assert_eq!(row_count(&s), 1, "upsert must not create a duplicate row");
    }

    #[test]
    fn large_blob_roundtrip() {
        let mut s = SqliteStorage::in_memory().unwrap();
        let blob: Vec<u8> = (0_u32..1024 * 1024).map(|i| (i % 251) as u8).collect();
        s.save_snapshot("big", &blob).unwrap();
        let got = s.load_snapshot("big").unwrap().expect("blob present");
        assert_eq!(got.len(), blob.len());
        assert_eq!(got, blob);
    }

    #[test]
    fn file_based_persists_across_reopen() {
        let path = temp_path("persist");
        {
            let mut s = SqliteStorage::open(&path).unwrap();
            s.save_snapshot("k", b"persisted").unwrap();
            // s drops here, closing the connection.
        }
        let s = SqliteStorage::open(&path).unwrap();
        let got = s.load_snapshot("k").unwrap();
        assert_eq!(got.as_deref(), Some(&b"persisted"[..]));

        drop(s);
        // Cleanup is best-effort ; ignore errors on Windows where WAL
        // sidecar files may briefly linger.
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn open_creates_parent_directory() {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let root = std::env::temp_dir().join(format!("bse-storage-{pid}-{nanos}-mkdirs"));
        let nested = root.join("a").join("b").join("c").join("snapshots.sqlite");
        assert!(!nested.parent().unwrap().exists());

        let mut s = SqliteStorage::open(&nested).unwrap();
        s.save_snapshot("k", b"v").unwrap();
        assert_eq!(s.load_snapshot("k").unwrap().as_deref(), Some(&b"v"[..]));

        drop(s);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn wal_mode_is_enabled_for_file_based() {
        let path = temp_path("wal");
        let s = SqliteStorage::open(&path).unwrap();
        let mode: String = s
            .lock()
            .unwrap()
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode.to_ascii_lowercase(), "wal");
        drop(s);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    /// Compile-time check that `SqliteStorage` satisfies the trait bounds.
    #[test]
    fn impls_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SqliteStorage>();
    }
}
