//! Saw blade component for rotating circular hazards.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A rotating saw blade that deals damage on contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SawBladeComponent {
    /// Damage dealt on contact.
    pub damage: i32,
    /// Rotation speed in radians per second.
    pub rotation_speed: f32,
    /// Orbit radius in pixels (0 = stationary rotation).
    pub radius: f32,
    /// Current angle in radians.
    pub current_angle: f32,
}

impl Default for SawBladeComponent {
    fn default() -> Self {
        Self {
            damage: 15,
            rotation_speed: 4.0,
            radius: 0.0,
            current_angle: 0.0,
        }
    }
}

impl Component for SawBladeComponent {}
