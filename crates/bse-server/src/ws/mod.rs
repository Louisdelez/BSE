//! WebSocket handling for `/ws/rooms/:room_id`.
//!
//! Each connection joins the [`crate::rooms::RoomManager`] on upgrade.
//! Inbound binary / text frames are broadcast to every other peer in
//! the same room (v010.2).

pub mod upgrade;

pub use upgrade::ws_room;
