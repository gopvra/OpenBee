//! Main game application entry point for OpenBee.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::game_logic::GameLogic;

/// Difficulty settings affecting enemy health, damage, and pickup rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Normal
    }
}

/// Game configuration loaded from settings file or command-line arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub language: String,
    pub difficulty: Difficulty,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 600,
            fullscreen: false,
            vsync: true,
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 0.8,
            language: "en".to_string(),
            difficulty: Difficulty::Normal,
        }
    }
}

/// Top-level game application managing initialization, the main loop frame, and shutdown.
pub struct BeeGameApp {
    pub game_logic: GameLogic,
    pub config: GameConfig,
    pub running: bool,
}

impl BeeGameApp {
    /// Create a new game application with the given configuration.
    pub fn new(config: GameConfig) -> Result<Self> {
        let game_logic = GameLogic::new(config.difficulty);
        Ok(Self {
            game_logic,
            config,
            running: false,
        })
    }

    /// Initialize all subsystems (renderer, audio, physics, etc.).
    pub fn initialize(&mut self) -> Result<()> {
        info!(
            "Initializing OpenBee {}x{} fullscreen={} vsync={}",
            self.config.window_width, self.config.window_height,
            self.config.fullscreen, self.config.vsync
        );
        self.game_logic.initialize()?;
        self.running = true;
        info!("OpenBee initialized successfully");
        Ok(())
    }

    /// Advance one frame of the game with the given delta time in seconds.
    pub fn run_frame(&mut self, dt: f64) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        self.game_logic.update(dt as f32)?;
        Ok(())
    }

    /// Shut down all subsystems and release resources.
    pub fn shutdown(&mut self) {
        info!("Shutting down OpenBee");
        self.running = false;
        self.game_logic.shutdown();
    }
}
