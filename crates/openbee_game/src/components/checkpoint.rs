//! Checkpoint component for respawn points.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A checkpoint that the player can activate for respawning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointComponent {
    /// Unique identifier for this checkpoint within the level.
    pub checkpoint_id: u32,
    /// Whether the player has activated this checkpoint.
    pub activated: bool,
    /// X coordinate of the respawn position.
    pub respawn_x: f32,
    /// Y coordinate of the respawn position.
    pub respawn_y: f32,
}

impl Default for CheckpointComponent {
    fn default() -> Self {
        Self {
            checkpoint_id: 0,
            activated: false,
            respawn_x: 0.0,
            respawn_y: 0.0,
        }
    }
}

impl Component for CheckpointComponent {}
