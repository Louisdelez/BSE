//! JWT issuance and verification.
//!
//! v016 uses **HS256** (HMAC-SHA-256) with a shared secret.  This
//! keeps the dependency surface minimal and is appropriate while
//! BSE runs as a single server process : the same binary both
//! issues and validates tokens, so a symmetric key is sufficient.
//!
//! When multi-instance / federated deployments land in **v016.1**
//! we expect to swap to **RS256** (asymmetric signing) so that
//! verifiers no longer need to hold the signing secret.  Migration
//! is straightforward : add a new `algorithm` field to [`JwtConfig`]
//! and dispatch on it in [`JwtConfig::issue`] / [`JwtConfig::verify`].

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
    errors::ErrorKind,
};

use crate::AuthError;
use crate::claims::{Claims, DEFAULT_ACCESS_TTL, DEFAULT_REFRESH_TTL, TokenType};

/// Configuration shared by token issuance and verification.
///
/// In a normal deployment one instance of this struct is built at
/// boot from environment variables and reused for the lifetime of
/// the process.  It is `Clone` so each request handler can hold its
/// own copy without locking.
#[derive(Clone)]
pub struct JwtConfig {
    /// HS256 shared secret.  Should be at least 32 bytes of CSPRNG
    /// output.  Treat as highly sensitive : anyone with this value
    /// can mint valid tokens.
    pub secret: Vec<u8>,
    /// Value placed in the `iss` claim ; verifiers reject tokens
    /// whose `iss` does not match.
    pub issuer: String,
    /// Lifetime of access tokens minted via [`JwtConfig::issue`]
    /// with [`TokenType::Access`].
    pub access_ttl: Duration,
    /// Lifetime of refresh tokens minted via [`JwtConfig::issue`]
    /// with [`TokenType::Refresh`].
    pub refresh_ttl: Duration,
}

impl std::fmt::Debug for JwtConfig {
    // Hand-rolled to avoid leaking the secret bytes in logs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtConfig")
            .field(
                "secret",
                &format_args!("<{} bytes redacted>", self.secret.len()),
            )
            .field("issuer", &self.issuer)
            .field("access_ttl", &self.access_ttl)
            .field("refresh_ttl", &self.refresh_ttl)
            .finish()
    }
}

impl JwtConfig {
    /// Build a [`JwtConfig`] with the default TTLs.
    ///
    /// `secret` should be at least 32 bytes of cryptographically
    /// random data — short or low-entropy secrets are not rejected
    /// here (HS256 accepts any byte length), but the resulting
    /// tokens will be trivially forgeable.
    #[must_use]
    pub fn new(secret: impl Into<Vec<u8>>, issuer: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            issuer: issuer.into(),
            access_ttl: DEFAULT_ACCESS_TTL,
            refresh_ttl: DEFAULT_REFRESH_TTL,
        }
    }

    /// Issue a signed JWT for the given subject.
    ///
    /// `iat` is set to the current system time and `exp` is `iat`
    /// plus the appropriate TTL for `kind`.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] if the system clock is before
    /// the UNIX epoch (extremely unlikely), and [`AuthError::InvalidToken`]
    /// if the JWT library fails to encode the payload (should not
    /// happen with well-formed claims).
    pub fn issue(
        &self,
        sub: &str,
        email: Option<&str>,
        name: Option<&str>,
        kind: TokenType,
    ) -> Result<String, AuthError> {
        let now = now_unix()?;
        let ttl = match kind {
            TokenType::Access => self.access_ttl,
            TokenType::Refresh => self.refresh_ttl,
        };
        let exp = now
            .checked_add(ttl.as_secs())
            .ok_or_else(|| AuthError::Internal("exp overflow".into()))?;

        let claims = Claims {
            sub: sub.to_owned(),
            email: email.map(str::to_owned),
            name: name.map(str::to_owned),
            iat: now,
            exp,
            iss: self.issuer.clone(),
            token_type: kind,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )
        .map_err(|err| AuthError::InvalidToken(err.to_string()))
    }

    /// Verify a token's signature and `exp`, return its [`Claims`].
    ///
    /// The `iss` claim must match this config's [`Self::issuer`] ;
    /// the `exp` claim is enforced automatically by `jsonwebtoken`
    /// and surfaces as [`AuthError::Expired`].
    ///
    /// # Errors
    ///
    /// - [`AuthError::Expired`] if the token's `exp` is in the past.
    /// - [`AuthError::InvalidToken`] for any other failure : bad
    ///   signature, malformed payload, wrong issuer, wrong algorithm.
    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        // `exp` validation is on by default ; restate it explicitly
        // so a future change in defaults does not silently weaken us.
        validation.validate_exp = true;

        let data: TokenData<Claims> =
            decode(token, &DecodingKey::from_secret(&self.secret), &validation).map_err(|err| {
                match err.kind() {
                    ErrorKind::ExpiredSignature => AuthError::Expired,
                    _ => AuthError::InvalidToken(err.to_string()),
                }
            })?;
        Ok(data.claims)
    }
}

/// Current UNIX time in seconds, as a non-negative `u64`.
fn now_unix() -> Result<u64, AuthError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| AuthError::Internal(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(secret: &[u8]) -> JwtConfig {
        JwtConfig::new(secret.to_vec(), "bse")
    }

    #[test]
    fn issue_and_verify_roundtrip() {
        let c = cfg(b"this-is-a-32-byte-test-secret-ok!");
        let token = c
            .issue("user-1", Some("a@b.c"), Some("Alice"), TokenType::Access)
            .expect("issue");
        let claims = c.verify(&token).expect("verify");
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.email.as_deref(), Some("a@b.c"));
        assert_eq!(claims.name.as_deref(), Some("Alice"));
        assert_eq!(claims.iss, "bse");
        assert_eq!(claims.token_type, TokenType::Access);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn verify_with_wrong_secret_fails() {
        let signer = cfg(b"secret-A-secret-A-secret-A-secret");
        let token = signer
            .issue("user-1", None, None, TokenType::Access)
            .expect("issue");
        let verifier = cfg(b"secret-B-secret-B-secret-B-secret");
        let err = verifier.verify(&token).expect_err("must fail");
        assert!(matches!(err, AuthError::InvalidToken(_)));
    }

    #[test]
    fn verify_wrong_issuer_fails() {
        let c = cfg(b"some-secret-some-secret-some-secret");
        let token = c
            .issue("user-1", None, None, TokenType::Access)
            .expect("issue");
        let other = JwtConfig::new(c.secret.clone(), "not-bse");
        let err = other.verify(&token).expect_err("must fail");
        assert!(matches!(err, AuthError::InvalidToken(_)));
    }

    #[test]
    fn refresh_ttl_is_longer_than_access_ttl() {
        let c = cfg(b"any-secret-any-secret-any-secret!");
        let access = c
            .issue("u", None, None, TokenType::Access)
            .and_then(|t| c.verify(&t))
            .expect("access ok");
        let refresh = c
            .issue("u", None, None, TokenType::Refresh)
            .and_then(|t| c.verify(&t))
            .expect("refresh ok");
        assert!(refresh.exp > access.exp);
    }

    #[test]
    fn expired_token_is_rejected() {
        // `jsonwebtoken` applies a 60-second `exp` leeway by default,
        // which is too long to wait in a unit test.  We hand-craft a
        // token whose `exp` is well in the past and check that our
        // wrapper reports it as [`AuthError::Expired`].
        let c = cfg(b"expiry-test-secret-expiry-test!!!");
        let now = now_unix().expect("clock");
        let expired = Claims {
            sub: "u".into(),
            email: None,
            name: None,
            iat: now - 10_000,
            exp: now - 5_000,
            iss: "bse".into(),
            token_type: TokenType::Access,
        };
        let raw = encode(
            &Header::new(Algorithm::HS256),
            &expired,
            &EncodingKey::from_secret(&c.secret),
        )
        .expect("encode expired");
        let auth_err = c.verify(&raw).expect_err("verify must fail");
        assert!(matches!(auth_err, AuthError::Expired));
    }

    #[test]
    fn short_ttl_token_expires_after_sleep() {
        // Live-clock check : issue a 2-second token and sleep past
        // its expiry, then bypass the public `verify` to use
        // `leeway = 0` and confirm the underlying library reports
        // expiry.  Our public `verify` would also reject given the
        // default 60s leeway, but waiting that long is impractical.
        let mut c = cfg(b"short-ttl-secret-short-ttl-secret");
        c.access_ttl = Duration::from_secs(2);
        let token = c.issue("u", None, None, TokenType::Access).expect("issue");
        std::thread::sleep(Duration::from_millis(3100));

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["bse"]);
        validation.leeway = 0;
        let err = jsonwebtoken::decode::<Claims>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(&c.secret),
            &validation,
        )
        .expect_err("must be expired with zero leeway");
        assert_eq!(err.kind(), &ErrorKind::ExpiredSignature);
    }
}
