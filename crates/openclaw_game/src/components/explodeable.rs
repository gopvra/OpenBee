//! Explodeable component for entities that detonate on destruction.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// An entity that creates an explosion dealing area damage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplodeableComponent {
    /// Explosion radius in pixels.
    pub explosion_radius: f32,
    /// Damage dealt to entities within the radius.
    pub damage: i32,
    /// Knockback force applied to entities within the radius.
    pub force: f32,
    /// Whether to spawn particle effects.
    pub particles: bool,
}

impl Default for ExplodeableComponent {
    fn default() -> Self {
        Self {
            explosion_radius: 64.0,
            damage: 10,
            force: 300.0,
            particles: true,
        }
    }
}

impl Component for ExplodeableComponent {}
