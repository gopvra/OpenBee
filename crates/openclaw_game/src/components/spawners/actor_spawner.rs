//! Actor spawner component for periodically creating enemies or items.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Spawns actors from a template at regular intervals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorSpawnerComponent {
    /// Template name of the actor to spawn.
    pub template: String,
    /// Time between spawns in seconds.
    pub spawn_interval: f32,
    /// Maximum number of spawned actors alive at once.
    pub max_alive: u32,
    /// Total number of actors spawned so far.
    pub spawn_count: u32,
    /// Current timer until next spawn.
    pub timer: f32,
}

impl Default for ActorSpawnerComponent {
    fn default() -> Self {
        Self {
            template: String::new(),
            spawn_interval: 5.0,
            max_alive: 3,
            spawn_count: 0,
            timer: 0.0,
        }
    }
}

impl Component for ActorSpawnerComponent {}
