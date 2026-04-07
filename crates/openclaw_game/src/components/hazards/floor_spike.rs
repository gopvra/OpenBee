//! Floor spike component for toggling spike traps.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// State of a floor spike trap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpikeState {
    /// Spikes are retracted and safe.
    Retracted,
    /// Spikes are extending upward.
    Extending,
    /// Spikes are fully extended and dangerous.
    Extended,
    /// Spikes are retracting back into the floor.
    Retracting,
}

/// A floor spike trap that cycles between retracted and extended states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorSpikeComponent {
    /// Damage dealt when the player contacts extended spikes.
    pub damage: i32,
    /// Whether the spikes are currently capable of dealing damage.
    pub active: bool,
    /// Total cycle time in seconds (retract + extend).
    pub cycle_time: f32,
    /// Current timer within the cycle.
    pub timer: f32,
    /// Current state of the spike.
    pub state: SpikeState,
}

impl Default for FloorSpikeComponent {
    fn default() -> Self {
        Self {
            damage: 10,
            active: true,
            cycle_time: 2.0,
            timer: 0.0,
            state: SpikeState::Retracted,
        }
    }
}

impl Component for FloorSpikeComponent {}
