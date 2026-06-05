//! Authentication primitives for BSE.
//!
//! This crate provides the small, self-contained building blocks
//! used by higher layers (the server's HTTP handlers, the desktop
//! client's login flow) to authenticate users :
//!
//! - [`hash_password`] / [`verify_password`] — Argon2id, OWASP
//!   default parameters, PHC-encoded strings.
//! - [`JwtConfig`], [`Claims`], [`TokenType`] — HS256 JWT issuance
//!   and verification.  See [`jwt`] for why HS256 and not RS256.
//! - [`SessionState`] — a tiny enum tracking whether a client is
//!   currently signed in.
//! - [`AuthError`] — a single unified error type for all of the
//!   above.
//!
//! The crate has **no async runtime** and no I/O : it is a pure
//! library of cryptographic helpers.  Persistence of users, tokens,
//! revocation lists, etc. is the job of other crates (`bse-storage`,
//! `bse-server`) that consume these primitives.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod claims;
pub mod error;
pub mod jwt;
pub mod password;
pub mod session;

pub use claims::{Claims, DEFAULT_ACCESS_TTL, DEFAULT_REFRESH_TTL, TokenType};
pub use error::AuthError;
pub use jwt::JwtConfig;
pub use password::{hash_password, verify_password};
pub use session::SessionState;
