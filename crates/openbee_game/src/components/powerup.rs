//! Powerup component for timed ability boosts.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Types of powerups available in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerupType {
    /// Immune to all damage.
    Invincibility,
    /// Sword deals fire damage with extended range.
    FireSword,
    /// Sword deals ice damage, freezes enemies.
    IceSword,
    /// Sword deals lightning damage, chains to nearby enemies.
    LightningSword,
    /// Increased movement speed and jump height.
    Catnip,
    /// Invisible to enemies, can pass through hazards.
    Ghost,
}

/// A timed powerup effect on an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerupComponent {
    /// Type of the active powerup.
    pub powerup_type: PowerupType,
    /// Total duration in seconds.
    pub duration: f32,
    /// Time remaining in seconds.
    pub remaining: f32,
    /// Whether the powerup is currently active.
    pub active: bool,
}

impl PowerupComponent {
    /// Create a new powerup with the given type and duration.
    pub fn new(powerup_type: PowerupType, duration: f32) -> Self {
        Self {
            powerup_type,
            duration,
            remaining: duration,
            active: true,
        }
    }

    /// Tick the powerup timer. Returns true if the powerup has expired.
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active {
            return true;
        }
        self.remaining -= dt;
        if self.remaining <= 0.0 {
            self.active = false;
            self.remaining = 0.0;
            true
        } else {
            false
        }
    }

    /// Fraction of duration remaining (0.0 to 1.0).
    pub fn fraction_remaining(&self) -> f32 {
        if self.duration <= 0.0 {
            return 0.0;
        }
        self.remaining / self.duration
    }
}

impl Component for PowerupComponent {}
