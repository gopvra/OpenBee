//! Lobby system — room management, chat, ready-state tracking.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::protocol::{ClientId, NetworkMessage, PlayerInfo};

/// Represents a single lobby room on the server.
#[derive(Debug)]
pub struct Lobby {
    /// Room name / title.
    pub name: String,
    /// Map / level name that will be played.
    pub level: String,
    /// Maximum number of players.
    pub max_players: u32,
    /// Currently joined players.
    pub players: HashMap<ClientId, LobbyPlayer>,
    /// Chat message history (most recent last).
    pub chat_history: Vec<ChatEntry>,
    /// Whether the game has been started.
    pub game_started: bool,
    /// ID of the hosting player (first to join).
    pub host_id: Option<ClientId>,
}

/// Per-player lobby state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub id: ClientId,
    pub name: String,
    pub ready: bool,
}

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEntry {
    pub sender_id: ClientId,
    pub sender_name: String,
    pub message: String,
    pub timestamp_ms: u64,
}

impl Lobby {
    /// Create a new empty lobby.
    pub fn new(name: impl Into<String>, level: impl Into<String>, max_players: u32) -> Self {
        Self {
            name: name.into(),
            level: level.into(),
            max_players,
            players: HashMap::new(),
            chat_history: Vec::new(),
            game_started: false,
            host_id: None,
        }
    }

    /// Add a player to the lobby. Returns `false` if the lobby is full.
    pub fn add_player(&mut self, id: ClientId, name: impl Into<String>) -> bool {
        if self.players.len() as u32 >= self.max_players {
            return false;
        }
        let name = name.into();
        info!("Player '{name}' (id={id}) joined lobby '{}'", self.name);
        if self.host_id.is_none() {
            self.host_id = Some(id);
        }
        self.players.insert(
            id,
            LobbyPlayer {
                id,
                name,
                ready: false,
            },
        );
        true
    }

    /// Remove a player from the lobby.
    pub fn remove_player(&mut self, id: ClientId) {
        if let Some(p) = self.players.remove(&id) {
            info!("Player '{}' (id={id}) left lobby '{}'", p.name, self.name);
        }
        // Migrate host.
        if self.host_id == Some(id) {
            self.host_id = self.players.keys().next().copied();
        }
    }

    /// Toggle or set the ready state for a player.
    pub fn set_ready(&mut self, id: ClientId, ready: bool) {
        if let Some(player) = self.players.get_mut(&id) {
            player.ready = ready;
            debug!("Player {} ready={}", player.name, ready);
        }
    }

    /// Check whether all players are ready (and there is at least one player).
    pub fn all_ready(&self) -> bool {
        !self.players.is_empty() && self.players.values().all(|p| p.ready)
    }

    /// Record a chat message.
    pub fn add_chat(&mut self, sender_id: ClientId, message: impl Into<String>, timestamp_ms: u64) {
        let sender_name = self
            .players
            .get(&sender_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        self.chat_history.push(ChatEntry {
            sender_id,
            sender_name,
            message: message.into(),
            timestamp_ms,
        });
    }

    /// Build a `LobbyUpdate` network message from the current state.
    pub fn to_lobby_update(&self) -> NetworkMessage {
        let players: Vec<PlayerInfo> = self
            .players
            .values()
            .map(|p| PlayerInfo {
                id: p.id,
                name: p.name.clone(),
                ready: p.ready,
            })
            .collect();
        NetworkMessage::LobbyUpdate { players }
    }

    /// Start the game if the caller is the host and everyone is ready.
    pub fn try_start(&mut self, requester: ClientId) -> Option<NetworkMessage> {
        if self.host_id != Some(requester) {
            return None;
        }
        if !self.all_ready() {
            return None;
        }
        self.game_started = true;
        info!("Game started in lobby '{}'", self.name);
        Some(NetworkMessage::StartGame)
    }
}
