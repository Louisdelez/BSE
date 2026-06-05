//! In-memory client-side session state.
//!
//! [`SessionState`] is a simple, cloneable value that a UI layer can
//! hold to know whether the user is currently signed in and, if so,
//! which tokens to present on outgoing requests.  It deliberately
//! does **not** know how to refresh itself : that policy lives in
//! a higher-level "auth service" layer, which mutates this value as
//! sign-in / sign-out / refresh transitions complete.

use bse_types::ids::UserId;

/// Whether a client is currently authenticated, and if so the
/// material it needs to act on the user's behalf.
///
/// The default is [`SessionState::SignedOut`], chosen so that types
/// embedding `SessionState` (e.g. an application model) start in a
/// safe, anonymous state without extra ceremony.
#[derive(Clone, Debug, Default)]
pub enum SessionState {
    /// No user is signed in.  Outgoing requests should omit the
    /// `Authorization` header and the UI should show the sign-in
    /// screen.
    #[default]
    SignedOut,
    /// A user has signed in and the client holds valid (or at least
    /// not-yet-known-expired) tokens.
    SignedIn {
        /// Identifier of the signed-in user.
        user_id: UserId,
        /// Human-readable name displayed in the UI.
        display_name: String,
        /// Bearer token sent on API calls.
        access_token: String,
        /// Token used to mint a new access token when the current
        /// one expires.
        refresh_token: String,
        /// UNIX seconds at which `access_token` stops being valid.
        /// The client uses this to proactively refresh shortly
        /// before expiry rather than reacting to a 401.
        access_expires_at: u64,
    },
}

impl SessionState {
    /// `true` iff this state carries credentials.
    #[must_use]
    pub fn is_signed_in(&self) -> bool {
        matches!(self, Self::SignedIn { .. })
    }

    /// The signed-in user's identifier, if any.
    #[must_use]
    pub fn user_id(&self) -> Option<UserId> {
        match self {
            Self::SignedOut => None,
            Self::SignedIn { user_id, .. } => Some(*user_id),
        }
    }

    /// Borrow the current access token, if signed in.
    ///
    /// Provided for convenience so call sites do not have to
    /// destructure the enum just to attach an `Authorization`
    /// header.
    #[must_use]
    pub fn access_token(&self) -> Option<&str> {
        match self {
            Self::SignedOut => None,
            Self::SignedIn { access_token, .. } => Some(access_token.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_signed_out() {
        let s = SessionState::default();
        assert!(!s.is_signed_in());
        assert!(s.user_id().is_none());
        assert!(s.access_token().is_none());
    }

    #[test]
    fn signed_in_helpers() {
        let uid = UserId::new();
        let s = SessionState::SignedIn {
            user_id: uid,
            display_name: "Alice".into(),
            access_token: "at".into(),
            refresh_token: "rt".into(),
            access_expires_at: 1_700_000_000,
        };
        assert!(s.is_signed_in());
        assert_eq!(s.user_id(), Some(uid));
        assert_eq!(s.access_token(), Some("at"));
    }
}
