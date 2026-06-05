//! In-memory user store backed by `bse-auth` for password verification.
//!
//! v016.1 stores users in a `RwLock<HashMap>` keyed by email. A future
//! milestone will move this to `PostgreSQL` via `bse-storage`. The whole
//! file is `~120` lines and lives behind a small API that the auth
//! handlers consume.

use std::collections::HashMap;
use std::sync::RwLock;

use bse_auth::{AuthError, hash_password, verify_password};
use bse_types::UserId;

/// One user record. The plaintext password is never stored, only the
/// Argon2id PHC hash.
#[derive(Clone, Debug)]
pub struct UserRecord {
    /// Stable identifier.
    pub id: UserId,
    /// Login email.
    pub email: String,
    /// Display name shown to other peers.
    pub display_name: String,
    /// Argon2id PHC hash of the password.
    pub password_hash: String,
}

/// In-memory user store.
#[derive(Default)]
pub struct UserStore {
    /// Keyed by lowercased email.
    by_email: RwLock<HashMap<String, UserRecord>>,
}

impl UserStore {
    /// Empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed a demo user at boot. Useful so the first run has a usable
    /// account without registering through the API. Idempotent : if
    /// the email is already present, nothing happens.
    ///
    /// Returns the user id of the seeded (or already-present) record.
    pub fn seed_if_empty(
        &self,
        email: &str,
        display_name: &str,
        password: &str,
    ) -> Result<UserId, AuthError> {
        {
            let map = self.by_email.read().expect("user store read");
            if let Some(existing) = map.get(&email.to_lowercase()) {
                return Ok(existing.id);
            }
        }
        let hash = hash_password(password)?;
        let id = UserId::new_v7();
        let record = UserRecord {
            id,
            email: email.to_string(),
            display_name: display_name.to_string(),
            password_hash: hash,
        };
        self.by_email
            .write()
            .expect("user store write")
            .insert(email.to_lowercase(), record);
        Ok(id)
    }

    /// Register a brand-new user. Returns `None` if the email is taken.
    pub fn register(
        &self,
        email: &str,
        display_name: &str,
        password: &str,
    ) -> Result<Option<UserId>, AuthError> {
        let key = email.to_lowercase();
        {
            let map = self.by_email.read().expect("user store read");
            if map.contains_key(&key) {
                return Ok(None);
            }
        }
        let hash = hash_password(password)?;
        let id = UserId::new_v7();
        let record = UserRecord {
            id,
            email: email.to_string(),
            display_name: display_name.to_string(),
            password_hash: hash,
        };
        self.by_email
            .write()
            .expect("user store write")
            .insert(key, record);
        Ok(Some(id))
    }

    /// Verify credentials. Returns the matching record on success.
    pub fn verify_credentials(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<UserRecord>, AuthError> {
        let map = self.by_email.read().expect("user store read");
        let Some(record) = map.get(&email.to_lowercase()) else {
            return Ok(None);
        };
        if verify_password(password, &record.password_hash)? {
            Ok(Some(record.clone()))
        } else {
            Ok(None)
        }
    }
}
