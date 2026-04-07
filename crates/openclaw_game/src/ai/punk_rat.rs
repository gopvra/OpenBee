//! PunkRat specialized AI: aggressive rat enemy with charge attack.

use glam::Vec2;
use openclaw_core::ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

/// PunkRat-specific AI states layered on top of base enemy AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PunkRatState {
    /// Idle, looking for trouble.
    Loitering,
    /// Spotted player, taunting before attack.
    Taunting,
    /// Winding up for a charge.
    WindUp,
    /// Charging at the player at high speed.
    Charging,
    /// Recovering after a charge.
    Recovering,
    /// Taking damage, recoiling.
    Hit,
}

/// Specialized AI for the PunkRat enemy, known for its charge attack.
#[derive(Debug, Clone)]
pub struct PunkRatAiComponent {
    /// Current PunkRat-specific state.
    pub state: PunkRatState,
    /// Charge speed (pixels per second).
    pub charge_speed: f32,
    /// Maximum charge distance (pixels).
    pub charge_distance: f32,
    /// Wind-up time before charging (seconds).
    pub windup_time: f32,
    /// Recovery time after charging (seconds).
    pub recovery_time: f32,
    /// Taunt duration (seconds).
    pub taunt_duration: f32,
    /// Timer for current state.
    pub state_timer: f32,
    /// Direction of the charge.
    pub charge_direction: Vec2,
    /// Distance charged so far.
    pub distance_charged: f32,
    /// The player entity being targeted.
    pub target: Option<Entity>,
}

impl PunkRatAiComponent {
    /// Create a new PunkRat AI.
    pub fn new() -> Self {
        Self {
            state: PunkRatState::Loitering,
            charge_speed: 500.0,
            charge_distance: 300.0,
            windup_time: 0.5,
            recovery_time: 1.0,
            taunt_duration: 0.8,
            state_timer: 0.0,
            charge_direction: Vec2::ZERO,
            distance_charged: 0.0,
            target: None,
        }
    }

    /// Update the PunkRat state machine.
    pub fn update(&mut self, dt: f32, target_position: Option<Vec2>, my_position: Vec2) {
        self.state_timer += dt;

        match self.state {
            PunkRatState::Loitering => {
                // Detect player
                if let Some(target_pos) = target_position {
                    let dist = (target_pos - my_position).length();
                    if dist < 300.0 {
                        self.state = PunkRatState::Taunting;
                        self.state_timer = 0.0;
                    }
                }
            }
            PunkRatState::Taunting => {
                if self.state_timer >= self.taunt_duration {
                    self.state = PunkRatState::WindUp;
                    self.state_timer = 0.0;
                    if let Some(target_pos) = target_position {
                        self.charge_direction = (target_pos - my_position).normalize_or_zero();
                    }
                }
            }
            PunkRatState::WindUp => {
                if self.state_timer >= self.windup_time {
                    self.state = PunkRatState::Charging;
                    self.state_timer = 0.0;
                    self.distance_charged = 0.0;
                }
            }
            PunkRatState::Charging => {
                let move_dist = self.charge_speed * dt;
                self.distance_charged += move_dist;
                if self.distance_charged >= self.charge_distance {
                    self.state = PunkRatState::Recovering;
                    self.state_timer = 0.0;
                }
            }
            PunkRatState::Recovering => {
                if self.state_timer >= self.recovery_time {
                    self.state = PunkRatState::Loitering;
                    self.state_timer = 0.0;
                }
            }
            PunkRatState::Hit => {
                if self.state_timer >= 0.5 {
                    self.state = PunkRatState::Recovering;
                    self.state_timer = 0.0;
                }
            }
        }
    }

    /// Get the velocity to apply during the charge.
    pub fn charge_velocity(&self) -> Vec2 {
        if self.state == PunkRatState::Charging {
            self.charge_direction * self.charge_speed
        } else {
            Vec2::ZERO
        }
    }

    /// Signal that the punk rat was hit, interrupting its current action.
    pub fn on_hit(&mut self) {
        self.state = PunkRatState::Hit;
        self.state_timer = 0.0;
    }
}

impl Default for PunkRatAiComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for PunkRatAiComponent {}
