//! JWT claim types and default lifetimes.
//!
//! Extracted from [`crate::jwt`] purely for file-size hygiene : these
//! types are referenced by both the issuance and verification paths
//! and have no behaviour of their own.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Default lifetime of an access token : 15 minutes.
///
/// Short on purpose : access tokens are bearer credentials, so a
/// stolen one should expire quickly.  Clients are expected to use
/// the refresh token (longer-lived, single-use in v016.1) to mint
/// a new access token when this one runs out.
pub const DEFAULT_ACCESS_TTL: Duration = Duration::from_secs(15 * 60);

/// Default lifetime of a refresh token : 30 days.
///
/// Long enough that users do not have to re-enter their password
/// during a normal usage cycle, short enough that a leaked token
/// becomes useless within a billing month.
pub const DEFAULT_REFRESH_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Discriminator stored in the JWT's `token_type` claim.
///
/// We embed the kind in the token itself (instead of relying on
/// separate signing keys) so that a single verification path can
/// reject e.g. an access token presented to the refresh endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Short-lived bearer token used for API calls.
    Access,
    /// Longer-lived token used solely to mint new access tokens.
    Refresh,
}

/// Claims serialized into the JWT payload.
///
/// Field names match the standard JWT registered claims
/// (`sub`, `iat`, `exp`, `iss`) so a third-party JWT debugger can
/// decode them without a custom schema.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject : the user identifier, stringified.
    pub sub: String,
    /// Optional email of the subject (for client-side display).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Optional display name of the subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Issued-at, UNIX seconds.
    pub iat: u64,
    /// Expiration, UNIX seconds.
    pub exp: u64,
    /// Issuer of the token (e.g. `"bse"`).
    pub iss: String,
    /// Whether this is an access or refresh token.
    pub token_type: TokenType,
}
