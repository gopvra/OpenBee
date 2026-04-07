//! Core game state management and main update loop.

use anyhow::Result;
use glam::Vec2;
use openclaw_core::ecs::World;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::game_app::Difficulty;
use crate::systems;

/// High-level game state controlling what the player sees and how input is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    MainMenu,
    Loading,
    Playing,
    Paused,
    BossIntro,
    BossFight,
    LevelComplete,
    GameOver,
    Victory,
    ScoreScreen,
}

impl Default for GameState {
    fn default() -> Self {
        Self::MainMenu
    }
}

/// A saved checkpoint within a level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub x: f32,
    pub y: f32,
    pub level: u32,
}

/// Central game logic coordinating the ECS world, game state, scoring, and level progression.
pub struct GameLogic {
    pub world: World,
    pub game_state: GameState,
    pub current_level: u32,
    pub lives: i32,
    pub score: u64,
    pub checkpoints: Vec<Checkpoint>,
    pub difficulty: Difficulty,
    accumulated_time: f32,
}

impl GameLogic {
    /// Fixed timestep for physics and logic updates (60 Hz).
    const FIXED_DT: f32 = 1.0 / 60.0;
    /// Maximum accumulated time to prevent spiral of death.
    const MAX_ACCUMULATION: f32 = 0.25;

    /// Create a new game logic instance with default state.
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            world: World::new(),
            game_state: GameState::MainMenu,
            current_level: 1,
            lives: 3,
            score: 0,
            checkpoints: Vec::new(),
            difficulty,
            accumulated_time: 0.0,
        }
    }

    /// Register all systems into the ECS world.
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing game logic, difficulty={:?}", self.difficulty);
        systems::register_all_systems(&mut self.world);
        Ok(())
    }

    /// Main update called each frame. Performs fixed-timestep updates.
    pub fn update(&mut self, dt: f32) -> Result<()> {
        match self.game_state {
            GameState::Playing | GameState::BossFight => {
                self.accumulated_time += dt.min(Self::MAX_ACCUMULATION);
                while self.accumulated_time >= Self::FIXED_DT {
                    self.world.update(Self::FIXED_DT);
                    self.accumulated_time -= Self::FIXED_DT;
                }
            }
            GameState::Loading => {
                // Loading is handled externally; just wait.
            }
            _ => {
                // Menus and non-gameplay states don't step physics.
            }
        }
        Ok(())
    }

    /// Transition to a new game state.
    pub fn set_state(&mut self, new_state: GameState) {
        debug!("Game state transition: {:?} -> {:?}", self.game_state, new_state);
        self.game_state = new_state;
    }

    /// Start a new game from level 1.
    pub fn new_game(&mut self) {
        info!("Starting new game");
        self.lives = match self.difficulty {
            Difficulty::Easy => 5,
            Difficulty::Normal => 3,
            Difficulty::Hard => 2,
        };
        self.score = 0;
        self.current_level = 1;
        self.checkpoints.clear();
        self.set_state(GameState::Loading);
    }

    /// Record a checkpoint at the given position.
    pub fn activate_checkpoint(&mut self, position: Vec2) {
        let cp = Checkpoint {
            x: position.x,
            y: position.y,
            level: self.current_level,
        };
        info!("Checkpoint activated at ({}, {}) level {}", cp.x, cp.y, cp.level);
        self.checkpoints.push(cp);
    }

    /// Get the most recent checkpoint for the current level, if any.
    pub fn last_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints
            .iter()
            .rev()
            .find(|cp| cp.level == self.current_level)
    }

    /// Handle player death: decrement lives and respawn or game over.
    pub fn player_died(&mut self) {
        self.lives -= 1;
        if self.lives <= 0 {
            info!("Game over - no lives remaining");
            self.set_state(GameState::GameOver);
        } else {
            info!("Player died, {} lives remaining", self.lives);
            // Respawn at last checkpoint handled by checkpoint system.
        }
    }

    /// Advance to the next level.
    pub fn complete_level(&mut self) {
        info!("Level {} complete!", self.current_level);
        self.set_state(GameState::ScoreScreen);
    }

    /// Move to the next level after the score screen.
    pub fn advance_to_next_level(&mut self) {
        self.current_level += 1;
        if self.current_level > 14 {
            self.set_state(GameState::Victory);
        } else {
            self.set_state(GameState::Loading);
        }
    }

    /// Add points to the score.
    pub fn add_score(&mut self, points: u64) {
        self.score += points;
    }

    /// Clean up resources.
    pub fn shutdown(&mut self) {
        info!("Game logic shutting down");
    }
}
