//! User store backed by [`crate::store::ServerStore`].
//!
//! This is a thin layer that turns the raw `SQLite` operations into the
//! verb-shaped API the auth handlers consume : register, login,
//! seed-on-empty. Passwords are hashed with Argon2id via `bse-auth`.

use std::sync::Arc;

use bse_auth::{AuthError, hash_password, verify_password};
use bse_types::UserId;
use thiserror::Error;
use tracing::warn;

use crate::store::{ServerStore, ServerStoreError, UserRow};

/// One user record returned from the store.
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

impl From<UserRow> for UserRecord {
    fn from(r: UserRow) -> Self {
        Self {
            id: r.id,
            email: r.email,
            display_name: r.display_name,
            password_hash: r.password_hash,
        }
    }
}

/// Combined error returned by user operations.
///
/// Either the password backend (`AuthError`) or the database
/// (`ServerStoreError`) can fail ; the handlers care about both.
#[derive(Debug, Error)]
pub enum UserStoreError {
    /// Password hashing / verification failed.
    #[error(transparent)]
    Auth(#[from] AuthError),
    /// Database backend failed.
    #[error(transparent)]
    Store(#[from] ServerStoreError),
}

/// Persistent user store.
#[derive(Clone)]
pub struct UserStore {
    store: Arc<ServerStore>,
}

impl UserStore {
    /// Wrap a shared [`ServerStore`].
    #[must_use]
    pub fn new(store: Arc<ServerStore>) -> Self {
        Self { store }
    }

    /// Seed a demo user at boot if (and only if) the table is empty.
    /// Idempotent across restarts thanks to the emptiness check.
    pub fn seed_if_empty(
        &self,
        email: &str,
        display_name: &str,
        password: &str,
    ) -> Result<(), UserStoreError> {
        if self.store.has_any_user()? {
            return Ok(());
        }
        let hash = hash_password(password)?;
        let inserted = self
            .store
            .insert_user(UserId::new_v7(), email, display_name, &hash)?;
        if !inserted {
            warn!(target: "bse::server::users", email, "seed user race lost ; ignoring");
        }
        Ok(())
    }

    /// Register a brand-new user. Returns `Ok(None)` if the email is
    /// already taken.
    pub fn register(
        &self,
        email: &str,
        display_name: &str,
        password: &str,
    ) -> Result<Option<UserId>, UserStoreError> {
        let hash = hash_password(password)?;
        let id = UserId::new_v7();
        let inserted = self.store.insert_user(id, email, display_name, &hash)?;
        if inserted { Ok(Some(id)) } else { Ok(None) }
    }

    /// Verify credentials. Returns the matching record on success.
    pub fn verify_credentials(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<UserRecord>, UserStoreError> {
        let Some(row) = self.store.user_by_email(email)? else {
            return Ok(None);
        };
        if verify_password(password, &row.password_hash)? {
            Ok(Some(row.into()))
        } else {
            Ok(None)
        }
    }
}
