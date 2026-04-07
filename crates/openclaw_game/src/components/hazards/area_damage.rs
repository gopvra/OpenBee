//! Area damage component for zones that periodically deal damage.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A hazardous area that deals periodic damage to entities within its radius.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaDamageComponent {
    /// Damage dealt each tick.
    pub damage: i32,
    /// Radius of the damage area in pixels.
    pub radius: f32,
    /// Time between damage ticks in seconds.
    pub interval: f32,
    /// Time accumulator since last damage tick.
    pub timer: f32,
}

impl Default for AreaDamageComponent {
    fn default() -> Self {
        Self {
            damage: 5,
            radius: 48.0,
            interval: 1.0,
            timer: 0.0,
        }
    }
}

impl Component for AreaDamageComponent {}
