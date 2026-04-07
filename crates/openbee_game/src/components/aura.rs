//! Aura component for visual glow effects around entities.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Types of aura visual effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuraType {
    /// Golden glow (treasures, powerups).
    Gold,
    /// Red glow (damage, fire).
    Red,
    /// Blue glow (ice, magic).
    Blue,
    /// Green glow (health, poison).
    Green,
    /// Purple glow (special items).
    Purple,
    /// White glow (invincibility).
    White,
}

/// A pulsing glow effect rendered around an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuraComponent {
    /// Type of aura to render.
    pub aura_type: AuraType,
    /// Radius of the glow in pixels.
    pub radius: f32,
    /// RGBA color of the aura.
    pub color: [u8; 4],
    /// Speed of the pulsing effect (cycles per second).
    pub pulse_speed: f32,
}

impl Default for AuraComponent {
    fn default() -> Self {
        Self {
            aura_type: AuraType::Gold,
            radius: 16.0,
            color: [255, 215, 0, 128],
            pulse_speed: 2.0,
        }
    }
}

impl Component for AuraComponent {}
