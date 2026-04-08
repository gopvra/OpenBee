//! Game server — accepts client connections, runs the simulation loop, and
//! broadcasts state updates.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};

use crate::protocol::{self, ClientId, EntitySnapshot, NetworkMessage, PlayerInfo, ServerInfo};

/// Configuration for starting a game server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address (e.g. `"0.0.0.0"`).
    pub address: String,
    /// TCP port.
    pub port: u16,
    /// Maximum number of concurrent players.
    pub max_players: u32,
    /// Server simulation tick rate in Hz.
    pub tick_rate: u32,
    /// Optional password required to connect.
    pub password: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".to_string(),
            port: 7777,
            max_players: 4,
            tick_rate: 30,
            password: None,
        }
    }
}

/// Per-client connection state held by the server.
pub struct ClientConnection {
    pub player_info: PlayerInfo,
    pub writer: Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
}

/// Shared mutable game state accessible from async tasks.
pub type SharedGameState = Arc<RwLock<GameState>>;

/// Server-authoritative game state.
#[derive(Debug, Default)]
pub struct GameState {
    pub tick: u64,
    pub entity_snapshots: Vec<EntitySnapshot>,
    pub running: bool,
}

/// The multiplayer game server.
pub struct GameServer {
    pub config: ServerConfig,
    pub clients: Arc<RwLock<HashMap<ClientId, ClientConnection>>>,
    pub game_state: SharedGameState,
    pub tick_rate: u32,
    next_client_id: Arc<Mutex<ClientId>>,
    listener: Option<TcpListener>,
}

impl GameServer {
    /// Bind and start listening.  Does **not** block — call [`tick`] in a loop.
    pub async fn start(config: ServerConfig) -> Result<Self> {
        let addr = format!("{}:{}", config.address, config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {addr}");

        let tick_rate = config.tick_rate;
        Ok(Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            game_state: Arc::new(RwLock::new(GameState::default())),
            tick_rate,
            next_client_id: Arc::new(Mutex::new(1)),
            listener: Some(listener),
        })
    }

    /// Accept one pending connection (non-blocking via `try_accept`).
    pub async fn accept_pending(&self) -> Result<()> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Server not started"))?;

        // Accept an incoming connection.
        let (stream, addr) = listener.accept().await?;
        stream.set_nodelay(true)?;

        info!("New connection from {addr}");

        let (reader, writer) = tokio::io::split(stream);
        let _ = reader; // reading handled elsewhere

        let mut id_guard = self.next_client_id.lock().await;
        let client_id = *id_guard;
        *id_guard += 1;
        drop(id_guard);

        let conn = ClientConnection {
            player_info: PlayerInfo {
                id: client_id,
                name: format!("Player{client_id}"),
                ready: false,
            },
            writer: Arc::new(Mutex::new(writer)),
        };

        // Send connect ack.
        let ack = NetworkMessage::ConnectAck {
            client_id,
            server_info: ServerInfo {
                name: "OpenBee Server".to_string(),
                level: "level1".to_string(),
                player_count: self.clients.read().await.len() as u32 + 1,
                max_players: self.config.max_players,
            },
        };
        let payload = protocol::encode(&ack)?;
        let framed = protocol::frame(&payload);
        conn.writer.lock().await.write_all(&framed).await?;

        self.clients.write().await.insert(client_id, conn);
        Ok(())
    }

    /// Run one server tick: advance simulation, broadcast state.
    pub async fn tick(&mut self) -> Result<()> {
        let mut state = self.game_state.write().await;
        state.tick += 1;

        // Build snapshot message.
        let snapshot = NetworkMessage::GameStateSnapshot {
            tick: state.tick,
            entities: state.entity_snapshots.clone(),
        };
        drop(state);

        self.broadcast(&snapshot).await?;
        Ok(())
    }

    /// Send a message to every connected client.
    pub async fn broadcast(&self, message: &NetworkMessage) -> Result<()> {
        let payload = protocol::encode(message)?;
        let framed = protocol::frame(&payload);

        let clients = self.clients.read().await;
        for (id, conn) in clients.iter() {
            let mut writer = conn.writer.lock().await;
            if let Err(e) = writer.write_all(&framed).await {
                warn!("Failed to send to client {id}: {e}");
            }
        }
        Ok(())
    }

    /// Gracefully shut down — notify all clients and stop listening.
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Server shutting down");
        let msg = NetworkMessage::Disconnect {
            reason: "Server shutting down".to_string(),
        };
        let _ = self.broadcast(&msg).await;
        self.clients.write().await.clear();
        self.listener = None;
        Ok(())
    }
}
