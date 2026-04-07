//! Life component for extra lives tracking.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Tracks remaining and maximum extra lives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeComponent {
    pub lives: i32,
    pub max_lives: i32,
}

impl LifeComponent {
    /// Add a life, clamped to maximum.
    pub fn add_life(&mut self) {
        self.lives = (self.lives + 1).min(self.max_lives);
    }

    /// Remove a life. Returns true if lives remain.
    pub fn lose_life(&mut self) -> bool {
        self.lives -= 1;
        self.lives > 0
    }
}

impl Default for LifeComponent {
    fn default() -> Self {
        Self {
            lives: 3,
            max_lives: 9,
        }
    }
}

impl Component for LifeComponent {}
