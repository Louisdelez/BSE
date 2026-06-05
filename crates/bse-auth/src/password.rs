//! Argon2id password hashing and verification.
//!
//! We use [`Argon2::default`], which currently selects the OWASP
//! recommended parameters for Argon2id (memory-hard, side-channel
//! resistant).  The output is a self-describing PHC string of the
//! form `$argon2id$v=19$m=...,t=...,p=...$<salt>$<hash>` so the
//! parameters can be upgraded later without a schema migration : a
//! hash produced today will still verify against any future default
//! because [`PasswordHash`] stores them inline.
//!
//! Verification is performed by the `argon2` crate, which compares
//! hashes in constant time, so a calling layer does **not** need to
//! add its own timing protection.

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

use crate::AuthError;

/// Hash a plaintext password using Argon2id with OWASP defaults.
///
/// The returned string is the full PHC representation
/// (`$argon2id$v=19$m=...$...$...`) and is safe to store as-is in a
/// database.  A fresh cryptographically random salt is generated for
/// every call, so hashing the same password twice produces two
/// different strings — that is by design.
///
/// # Errors
///
/// Returns [`AuthError::Hashing`] if the OS random number generator
/// fails, or if the Argon2 implementation rejects the parameters
/// (should not happen with the built-in defaults).
pub fn hash_password(plaintext: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(plaintext.as_bytes(), &salt)
        .map_err(|err| AuthError::Hashing(err.to_string()))?;
    Ok(hash.to_string())
}

/// Verify a plaintext password against a PHC hash string.
///
/// Returns `Ok(true)` if the password matches, `Ok(false)` if it
/// does not.  A malformed `hash` string is reported as
/// [`AuthError::Hashing`] rather than `Ok(false)` so that callers can
/// distinguish "wrong password" from "corrupted database row".
///
/// The comparison is constant-time, courtesy of the `argon2` crate.
///
/// # Errors
///
/// Returns [`AuthError::Hashing`] when `hash` is not a valid PHC
/// string or uses an algorithm we cannot evaluate.
pub fn verify_password(plaintext: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(hash).map_err(|err| AuthError::Hashing(err.to_string()))?;
    match Argon2::default().verify_password(plaintext.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(err) => Err(AuthError::Hashing(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("correct horse battery staple").expect("hash succeeds");
        assert!(hash.starts_with("$argon2id$"), "PHC prefix : got {hash}");
        let ok = verify_password("correct horse battery staple", &hash).expect("verify succeeds");
        assert!(ok, "matching password must verify");
    }

    #[test]
    fn verify_wrong_password_fails() {
        let hash = hash_password("hunter2").expect("hash succeeds");
        let ok = verify_password("hunter3", &hash).expect("verify completes");
        assert!(!ok, "mismatching password must not verify");
    }

    #[test]
    fn malformed_hash_is_an_error() {
        let err = verify_password("anything", "not-a-phc-string").expect_err("should fail");
        assert!(matches!(err, AuthError::Hashing(_)));
    }

    #[test]
    fn different_salts_produce_different_hashes() {
        let a = hash_password("same").expect("hash a");
        let b = hash_password("same").expect("hash b");
        assert_ne!(a, b, "fresh salt per call");
    }
}
