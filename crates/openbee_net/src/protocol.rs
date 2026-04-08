//! Network protocol — message types, serialisation helpers, and shared data
//! structures exchanged between client and server.

use serde::{Deserialize, Serialize};

/// Unique identifier for a connected client.
pub type ClientId = u32;

/// Top-level message enum covering every packet type in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    // -- Connection ----------------------------------------------------------
    /// Client requests to join the server.
    Connect {
        player_name: String,
        version: String,
    },
    /// Server acknowledges a successful connection.
    ConnectAck {
        client_id: ClientId,
        server_info: ServerInfo,
    },
    /// Either side terminates the connection.
    Disconnect { reason: String },
    /// Latency probe sent by either side.
    Ping { timestamp: u64 },
    /// Response to a [`Ping`].
    Pong { timestamp: u64 },

    // -- Lobby ---------------------------------------------------------------
    /// Broadcast of current lobby state.
    LobbyUpdate { players: Vec<PlayerInfo> },
    /// Chat message in the lobby or in-game.
    ChatMessage { sender: ClientId, message: String },
    /// Host signals the game to start.
    StartGame,

    // -- Gameplay ------------------------------------------------------------
    /// Client sends its input for a given simulation tick.
    PlayerInput {
        client_id: ClientId,
        tick: u64,
        input: PlayerInputData,
    },
    /// Authoritative full-world snapshot from the server.
    GameStateSnapshot {
        tick: u64,
        entities: Vec<EntitySnapshot>,
    },
    /// Incremental update for a single entity.
    EntityUpdate {
        entity_id: u64,
        components: Vec<ComponentUpdate>,
    },
    /// A new actor has been spawned.
    ActorSpawned {
        entity_id: u64,
        template: String,
        position: (f32, f32),
    },
    /// An actor has been destroyed.
    ActorDestroyed { entity_id: u64 },

    // -- Events --------------------------------------------------------------
    /// Generic game event forwarded over the wire.
    GameEvent { event_type: String, data: String },
}

/// Information about the server, sent in [`NetworkMessage::ConnectAck`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub level: String,
    pub player_count: u32,
    pub max_players: u32,
}

/// Public information about a player in the lobby.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: ClientId,
    pub name: String,
    pub ready: bool,
}

/// Compact input state sent each tick by a client.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerInputData {
    pub move_x: f32,
    pub jump: bool,
    pub attack: bool,
    pub use_item: bool,
}

/// Snapshot of a single entity's visual state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub entity_id: u64,
    pub x: f32,
    pub y: f32,
    pub animation: String,
    pub flip_x: bool,
}

/// Binary component data for an incremental entity update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUpdate {
    pub component_type: String,
    pub data: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Serialisation helpers
// ---------------------------------------------------------------------------

/// Serialise a message to a byte vector using bincode.
pub fn encode(msg: &NetworkMessage) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(msg)
}

/// Deserialise a message from bytes.
pub fn decode(data: &[u8]) -> Result<NetworkMessage, bincode::Error> {
    bincode::deserialize(data)
}

/// Wrap raw bytes with a 4-byte big-endian length prefix for framing.
pub fn frame(payload: &[u8]) -> Vec<u8> {
    let len = payload.len() as u32;
    let mut buf = Vec::with_capacity(4 + payload.len());
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(payload);
    buf
}
