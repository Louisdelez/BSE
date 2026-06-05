//! Error type for all `bse-auth` operations.
//!
//! A single [`AuthError`] enum covers password hashing, JWT issuance /
//! verification, and any other internal failures.  Callers typically
//! match on the variant to decide whether the failure is the caller's
//! fault (`InvalidToken`, `Expired`) or an internal bug (`Hashing`,
//! `Internal`).

use thiserror::Error;

/// All failures that can occur in `bse-auth`.
///
/// The string payloads carry a short human-readable reason ; they are
/// intentionally not structured because they exist mainly for logs and
/// developer diagnostics.  Do **not** expose them verbatim to end users
/// over the wire — they may leak implementation details (e.g. which
/// JWT field failed validation).
#[derive(Debug, Error)]
pub enum AuthError {
    /// Argon2 password hashing or verification failed for a reason
    /// other than "the password is wrong" (e.g. malformed hash string,
    /// OS RNG failure, unsupported algorithm in the stored PHC).
    #[error("password hashing failed : {0}")]
    Hashing(String),

    /// JWT signature, structure, or claim validation failed.  The
    /// inner string is the underlying library's reason ; treat it as
    /// opaque outside of logs.
    #[error("invalid token : {0}")]
    InvalidToken(String),

    /// JWT is structurally valid and correctly signed, but its `exp`
    /// claim is in the past.  Surfaced as a distinct variant so
    /// callers can trigger a refresh flow without inspecting strings.
    #[error("token expired")]
    Expired,

    /// Catch-all for unexpected errors that do not fit the categories
    /// above (e.g. a clock that travels backwards before the UNIX
    /// epoch).  Should be rare in practice.
    #[error("internal : {0}")]
    Internal(String),
}
