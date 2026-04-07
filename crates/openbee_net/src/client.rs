//! Game client — connects to a server, sends inputs, and receives state
//! updates.

use std::collections::VecDeque;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info};

use crate::protocol::{self, ClientId, NetworkMessage, PlayerInputData};

/// Configuration for connecting to a server.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub player_name: String,
    pub server_address: String,
    pub port: u16,
}

/// The client's connection lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    InLobby,
    InGame,
}

/// Multiplayer game client.
pub struct GameClient {
    pub config: ClientConfig,
    pub connection_state: ConnectionState,
    pub server_addr: String,
    pub player_id: Option<ClientId>,
    pub latency_ms: f64,
    pub outgoing_queue: VecDeque<NetworkMessage>,
    pub incoming_queue: VecDeque<NetworkMessage>,
    stream: Option<TcpStream>,
    last_ping_ts: u64,
}

impl GameClient {
    /// Create a new disconnected client.
    pub fn new(config: ClientConfig) -> Self {
        let server_addr = format!("{}:{}", config.server_address, config.port);
        Self {
            config,
            connection_state: ConnectionState::Disconnected,
            server_addr: server_addr.clone(),
            player_id: None,
            latency_ms: 0.0,
            outgoing_queue: VecDeque::new(),
            incoming_queue: VecDeque::new(),
            stream: None,
            last_ping_ts: 0,
        }
    }

    /// Initiate a TCP connection to the server and send a `Connect` message.
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to {}", self.server_addr);
        self.connection_state = ConnectionState::Connecting;

        let stream = TcpStream::connect(&self.server_addr).await?;
        stream.set_nodelay(true)?;
        self.stream = Some(stream);

        let connect_msg = NetworkMessage::Connect {
            player_name: self.config.player_name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        self.send(&connect_msg).await?;
        self.connection_state = ConnectionState::Connected;
        info!("Connected to server");
        Ok(())
    }

    /// Send a single message to the server.
    pub async fn send(&mut self, message: &NetworkMessage) -> Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Not connected"))?;
        let payload = protocol::encode(message)?;
        let framed = protocol::frame(&payload);
        stream.write_all(&framed).await?;
        Ok(())
    }

    /// Maximum allowed message size in bytes (16 MiB). Messages larger than
    /// this are rejected to prevent denial-of-service via memory exhaustion.
    const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

    /// Try to read one message from the server (non-blocking-ish).
    pub async fn receive(&mut self) -> Result<Option<NetworkMessage>> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Not connected"))?;

        // Read length prefix.
        let mut len_buf = [0u8; 4];
        match stream.try_read(&mut len_buf) {
            Ok(4) => {}
            Ok(_) => return Ok(None),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        // Reject oversized messages to prevent DoS via memory exhaustion.
        if len > Self::MAX_MESSAGE_SIZE {
            return Err(anyhow::anyhow!(
                "message size {} exceeds maximum allowed size {}",
                len,
                Self::MAX_MESSAGE_SIZE
            ));
        }

        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;
        let msg = protocol::decode(&buf)?;
        Ok(Some(msg))
    }

    /// Queue an input to be sent on the next flush.
    pub fn queue_input(&mut self, tick: u64, input: PlayerInputData) {
        if let Some(id) = self.player_id {
            self.outgoing_queue.push_back(NetworkMessage::PlayerInput {
                client_id: id,
                tick,
                input,
            });
        }
    }

    /// Flush all queued outgoing messages.
    pub async fn flush(&mut self) -> Result<()> {
        while let Some(msg) = self.outgoing_queue.pop_front() {
            self.send(&msg).await?;
        }
        Ok(())
    }

    /// Process a received message, updating local state as appropriate.
    pub fn handle_message(&mut self, msg: NetworkMessage) {
        match &msg {
            NetworkMessage::ConnectAck { client_id, .. } => {
                self.player_id = Some(*client_id);
                self.connection_state = ConnectionState::InLobby;
                info!("Assigned client ID {client_id}");
            }
            NetworkMessage::Disconnect { reason } => {
                info!("Disconnected: {reason}");
                self.connection_state = ConnectionState::Disconnected;
                self.stream = None;
            }
            NetworkMessage::Pong { timestamp } => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                self.latency_ms = (now.saturating_sub(*timestamp)) as f64;
                debug!("Latency: {:.1}ms", self.latency_ms);
            }
            NetworkMessage::StartGame => {
                self.connection_state = ConnectionState::InGame;
                info!("Game started!");
            }
            _ => {
                // Other messages go into the incoming queue for the game to process.
                self.incoming_queue.push_back(msg);
            }
        }
    }

    /// Disconnect gracefully.
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.stream.is_some() {
            let msg = NetworkMessage::Disconnect {
                reason: "Client disconnecting".to_string(),
            };
            let _ = self.send(&msg).await;
        }
        self.stream = None;
        self.connection_state = ConnectionState::Disconnected;
        self.player_id = None;
        info!("Disconnected");
        Ok(())
    }

    /// Send a ping to measure latency.
    pub async fn ping(&mut self) -> Result<()> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_ping_ts = ts;
        self.send(&NetworkMessage::Ping { timestamp: ts }).await
    }
}
