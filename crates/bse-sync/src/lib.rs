//! WebSocket sync client used by the BSE app to talk to a `bse-server`.
//!
//! The client speaks the [`bse_protocol`] wire format over a single
//! WebSocket per room and supports two parallel concerns :
//!
//! - **Ops** : binary CRDT updates, sent and received via
//!   [`SyncClient::send_op`] / [`SyncClient::next_message`].
//! - **Awareness** : ephemeral peer state (cursor, identity), throttled
//!   client-side by [`LocalCursor`] and decoded with [`decode_cursor`].
//!
//! See `docs/03-ARCHITECTURE/04-protocole-reseau.md` for the protocol
//! design, and the integration notes at the bottom of `client.rs` for
//! why there is no live-server test at this milestone.
//!
//! # Backward-compatible re-exports
//!
//! [`ConnectionState`] and [`SyncError`] keep the exact same public API
//! they had in the v002 stub : other crates (notably `bse-ui`) match on
//! their variants and embed [`ConnectionState::label`] in the UI.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod client;
mod cursor;
mod error;
mod peer;
mod state;

pub use client::{ClientConfig, SyncClient};
pub use cursor::LocalCursor;
pub use error::SyncError;
pub use peer::{PeerInfo, decode_cursor};
pub use state::ConnectionState;
