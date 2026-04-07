//! # OpenClaw Net
//!
//! Multiplayer networking for OpenClaw — client/server architecture with state
//! synchronisation, lobby management, and replay recording/playback.

pub mod client;
pub mod lobby;
pub mod protocol;
pub mod replay;
pub mod server;
pub mod sync;

pub use client::GameClient;
pub use protocol::{ClientId, NetworkMessage};
pub use server::GameServer;
