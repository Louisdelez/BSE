//! Server-side `SQLite` persistence (v018).
//!
//! Holds user accounts and per-room CRDT snapshots. The schema is
//! versioned through a `schema_migrations` table so the server can be
//! upgraded in-place across future versions without losing data.
//!
//! Concurrency model
//! -----------------
//! The underlying [`rusqlite::Connection`] is not `Sync`, so we wrap it
//! in a [`std::sync::Mutex`] and expose only the operations we actually
//! need. `SQLite` is in WAL mode so reads do not block writes.
//!
//! Storage path
//! ------------
//! The file lives at `{BSE_SERVER_DATA_DIR}/server.sqlite` (default
//! `./data/server.sqlite`). Use [`ServerStore::in_memory`] for tests.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use bse_types::UserId;
use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;
use tracing::{debug, info};

/// Errors produced by the server store.
#[derive(Debug, Error)]
pub enum ServerStoreError {
    /// Filesystem I/O failed.
    #[error("I/O error : {0}")]
    Io(String),

    /// Underlying `SQLite` operation failed.
    #[error("database error : {0}")]
    Database(String),

    /// A row referenced an id that could not be parsed as a `UserId`.
    #[error("invalid stored data : {0}")]
    Corruption(String),
}

impl From<std::io::Error> for ServerStoreError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<rusqlite::Error> for ServerStoreError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(err.to_string())
    }
}

/// A user record as stored in the `users` table.
#[derive(Clone, Debug)]
pub struct UserRow {
    /// Stable identifier.
    pub id: UserId,
    /// Login email (canonical lower-case form is stored separately for lookups).
    pub email: String,
    /// Display name shown to other peers.
    pub display_name: String,
    /// Argon2id PHC hash of the password.
    pub password_hash: String,
}

/// SQLite-backed server store.
///
/// Construct via [`ServerStore::open`] for a file-based database, or
/// [`ServerStore::in_memory`] for an ephemeral one (tests).
pub struct ServerStore {
    conn: Mutex<Connection>,
}

/// Schema migrations applied in order. Inserts into `schema_migrations`
/// happen inside the same transaction as the DDL.
const MIGRATIONS: &[(i64, &str)] = &[(
    1,
    "CREATE TABLE users (\
        id            TEXT PRIMARY KEY, \
        email         TEXT NOT NULL, \
        email_lc      TEXT NOT NULL UNIQUE, \
        display_name  TEXT NOT NULL, \
        password_hash TEXT NOT NULL, \
        created_at    INTEGER NOT NULL\
     ); \
     CREATE TABLE room_snapshots (\
        room_id     TEXT PRIMARY KEY, \
        bytes       BLOB NOT NULL, \
        updated_at  INTEGER NOT NULL\
     );",
)];

impl ServerStore {
    /// Open or create a SQLite-backed server store at `path`.
    pub fn open(path: &Path) -> Result<Self, ServerStoreError> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
            info!(parent = %parent.display(), "created server data directory");
        }
        let conn = Connection::open(path)?;
        debug!(path = %path.display(), "opened server store");
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.migrate()?;
        Ok(store)
    }

    /// Open an in-memory database. Useful for tests.
    pub fn in_memory() -> Result<Self, ServerStoreError> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), ServerStoreError> {
        let mut conn = self.lock()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (\
                version    INTEGER PRIMARY KEY, \
                applied_at INTEGER NOT NULL\
             );",
        )?;
        let applied: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        for (version, sql) in MIGRATIONS {
            if *version <= applied {
                continue;
            }
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                params![version, unix_now_secs()],
            )?;
            tx.commit()?;
            info!(version, "applied schema migration");
        }
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ServerStoreError> {
        self.conn
            .lock()
            .map_err(|_| ServerStoreError::Database("connection mutex poisoned".into()))
    }

    // ---------------------------- users ----------------------------

    /// Insert a user. Returns `Ok(false)` if the email is already taken.
    pub fn insert_user(
        &self,
        id: UserId,
        email: &str,
        display_name: &str,
        password_hash: &str,
    ) -> Result<bool, ServerStoreError> {
        let conn = self.lock()?;
        let res = conn.execute(
            "INSERT INTO users(id, email, email_lc, display_name, password_hash, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id.to_string(),
                email,
                email.to_lowercase(),
                display_name,
                password_hash,
                unix_now_secs(),
            ],
        );
        match res {
            Ok(_) => Ok(true),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
            {
                Ok(false)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Look up a user by email (case-insensitive). Returns `None` if
    /// the email is unknown.
    pub fn user_by_email(&self, email: &str) -> Result<Option<UserRow>, ServerStoreError> {
        let conn = self.lock()?;
        let row = conn
            .query_row(
                "SELECT id, email, display_name, password_hash FROM users WHERE email_lc = ?1",
                params![email.to_lowercase()],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;
        let Some((id, email, display_name, password_hash)) = row else {
            return Ok(None);
        };
        let id: UserId = id
            .parse()
            .map_err(|e: uuid::Error| ServerStoreError::Corruption(e.to_string()))?;
        Ok(Some(UserRow {
            id,
            email,
            display_name,
            password_hash,
        }))
    }

    /// `true` iff at least one user exists.
    pub fn has_any_user(&self) -> Result<bool, ServerStoreError> {
        let conn = self.lock()?;
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
        Ok(n > 0)
    }

    // ----------------------- room snapshots ------------------------

    /// Persist the latest known snapshot for `room_id`. Overwrites
    /// any existing row for that room.
    pub fn save_room_snapshot(
        &self,
        room_id: &str,
        bytes: &[u8],
    ) -> Result<(), ServerStoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO room_snapshots(room_id, bytes, updated_at) VALUES (?1, ?2, ?3) \
             ON CONFLICT(room_id) DO UPDATE SET bytes = excluded.bytes, updated_at = excluded.updated_at",
            params![room_id, bytes, unix_now_secs()],
        )?;
        Ok(())
    }

    /// Load the latest snapshot for `room_id`, if one was ever stored.
    pub fn load_room_snapshot(
        &self,
        room_id: &str,
    ) -> Result<Option<Vec<u8>>, ServerStoreError> {
        let conn = self.lock()?;
        let row = conn
            .query_row(
                "SELECT bytes FROM room_snapshots WHERE room_id = ?1",
                params![room_id],
                |r| r.get::<_, Vec<u8>>(0),
            )
            .optional()?;
        Ok(row)
    }
}

fn unix_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_is_idempotent() {
        let s = ServerStore::in_memory().unwrap();
        // Reopening an in-memory connection is not possible, but running
        // `migrate` again on the same handle must still be a no-op.
        s.migrate().unwrap();
        s.migrate().unwrap();
    }

    #[test]
    fn insert_user_then_lookup() {
        let s = ServerStore::in_memory().unwrap();
        let id = UserId::new();
        let ok = s
            .insert_user(id, "alice@example.com", "Alice", "phc$hash")
            .unwrap();
        assert!(ok);
        let row = s.user_by_email("Alice@Example.com").unwrap().unwrap();
        assert_eq!(row.id, id);
        assert_eq!(row.display_name, "Alice");
        assert_eq!(row.password_hash, "phc$hash");
    }

    #[test]
    fn duplicate_email_rejected() {
        let s = ServerStore::in_memory().unwrap();
        assert!(
            s.insert_user(UserId::new(), "a@b.c", "A", "h")
                .unwrap()
        );
        assert!(
            !s.insert_user(UserId::new(), "A@B.C", "A2", "h2")
                .unwrap()
        );
    }

    #[test]
    fn missing_user_returns_none() {
        let s = ServerStore::in_memory().unwrap();
        assert!(s.user_by_email("nobody@example.com").unwrap().is_none());
    }

    #[test]
    fn room_snapshot_roundtrip_and_overwrite() {
        let s = ServerStore::in_memory().unwrap();
        assert!(s.load_room_snapshot("lobby").unwrap().is_none());
        s.save_room_snapshot("lobby", b"first").unwrap();
        assert_eq!(
            s.load_room_snapshot("lobby").unwrap().as_deref(),
            Some(&b"first"[..])
        );
        s.save_room_snapshot("lobby", b"second").unwrap();
        assert_eq!(
            s.load_room_snapshot("lobby").unwrap().as_deref(),
            Some(&b"second"[..])
        );
    }

    #[test]
    fn has_any_user_tracks_inserts() {
        let s = ServerStore::in_memory().unwrap();
        assert!(!s.has_any_user().unwrap());
        s.insert_user(UserId::new(), "a@b.c", "A", "h").unwrap();
        assert!(s.has_any_user().unwrap());
    }
}
