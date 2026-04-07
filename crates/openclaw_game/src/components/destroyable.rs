//! Destroyable component for entities that can be destroyed with effects.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// An entity that plays an animation and/or sound and optionally spawns items upon destruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestroyableComponent {
    /// Whether the entity has been destroyed.
    pub is_destroyed: bool,
    /// Animation to play on destruction.
    pub destroy_animation: Option<String>,
    /// Sound to play on destruction.
    pub destroy_sound: Option<String>,
    /// Template names of actors/items to spawn on destruction.
    pub spawn_on_destroy: Vec<String>,
}

impl Default for DestroyableComponent {
    fn default() -> Self {
        Self {
            is_destroyed: false,
            destroy_animation: None,
            destroy_sound: None,
            spawn_on_destroy: Vec::new(),
        }
    }
}

impl Component for DestroyableComponent {}
