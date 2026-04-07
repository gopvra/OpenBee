//! Controllable component for entities that accept player input.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Flags controlling which actions the player can perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllableComponent {
    /// Whether this entity accepts input at all.
    pub is_active: bool,
    /// Whether the entity can move horizontally.
    pub can_move: bool,
    /// Whether the entity can jump.
    pub can_jump: bool,
    /// Whether the entity can attack.
    pub can_attack: bool,
}

impl Default for ControllableComponent {
    fn default() -> Self {
        Self {
            is_active: true,
            can_move: true,
            can_jump: true,
            can_attack: true,
        }
    }
}

impl Component for ControllableComponent {}
