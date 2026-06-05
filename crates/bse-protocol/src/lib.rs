//! Wire protocol shared between BSE client and server.
//!
//! Messages are transported over WebSocket as MessagePack-encoded payloads.
//! The structures defined here are the authoritative format ; both client
//! and server depend on this crate.
//!
//! See `docs/03-ARCHITECTURE/04-protocole-reseau.md` for the design
//! rationale and a packet-level walkthrough.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod messages;
mod version;

pub use messages::{
    AwarenessPayload, ClientMessage, ErrorCode, ErrorPayload, HelloPayload, OpPayload,
    PeerJoinPayload, PeerLeavePayload, ServerMessage, WelcomePayload,
};
pub use version::PROTOCOL_VERSION;
