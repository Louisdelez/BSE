//! WebSocket handling for `/ws/rooms/:room_id`.
//!
//! v008 ships a naive echo loop. A real room manager — wiring CRDT
//! ops, awareness, and persistence — lands in v009.

pub mod upgrade;

pub use upgrade::ws_room;
