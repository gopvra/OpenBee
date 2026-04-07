//! Stepping ground component for platforms that collapse when stood on.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A platform that begins falling after a delay when the player stands on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteppingGroundComponent {
    /// Delay before the platform starts falling (seconds).
    pub fall_delay: f32,
    /// Current timer since the player stepped on.
    pub timer: f32,
    /// Whether the platform is currently falling.
    pub falling: bool,
    /// Time before the platform resets to its original position (seconds; 0 = never).
    pub reset_time: f32,
    /// Original Y position for resetting.
    pub original_y: f32,
}

impl Default for SteppingGroundComponent {
    fn default() -> Self {
        Self {
            fall_delay: 0.5,
            timer: 0.0,
            falling: false,
            reset_time: 3.0,
            original_y: 0.0,
        }
    }
}

impl Component for SteppingGroundComponent {}
