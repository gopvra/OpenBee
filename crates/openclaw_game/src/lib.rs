//! # OpenClaw Game
//!
//! Game-specific logic for OpenClaw, a reimplementation of the Captain Claw (1997) game engine.
//! Contains all components, systems, AI behaviors, level management, and UI.

pub mod actor_controller;
pub mod actor_templates;
pub mod ai;
pub mod components;
pub mod game_app;
pub mod game_logic;
pub mod game_saves;
pub mod game_view;
pub mod levels;
pub mod systems;
pub mod ui;

pub use game_app::{ClawGameApp, Difficulty, GameConfig};
pub use game_logic::{Checkpoint, GameLogic, GameState};
