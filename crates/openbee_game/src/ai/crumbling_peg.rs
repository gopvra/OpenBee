//! Crumbling peg AI: a platform that crumbles after being stood on.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// State of a crumbling platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrumblePegState {
    /// Solid and stable.
    Idle,
    /// Being stood on, counting down to crumble.
    Crumbling,
    /// Has crumbled, no longer solid.
    Crumbled,
    /// Regenerating back to solid.
    Respawning,
}

/// AI component for platforms that crumble when stepped on and optionally regenerate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrumblingPegComponent {
    /// Current state.
    pub state: CrumblePegState,
    /// Time before the peg crumbles after being stepped on (seconds).
    pub crumble_delay: f32,
    /// Time before the peg respawns after crumbling (seconds; 0 = never).
    pub respawn_delay: f32,
    /// Current timer.
    pub timer: f32,
    /// The number of shakes/visual warnings before crumbling.
    pub shake_intensity: f32,
}

impl CrumblingPegComponent {
    /// Begin the crumble countdown when stepped on.
    pub fn start_crumble(&mut self) {
        if self.state == CrumblePegState::Idle {
            self.state = CrumblePegState::Crumbling;
            self.timer = 0.0;
        }
    }

    /// Update the crumbling peg state machine.
    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        match self.state {
            CrumblePegState::Idle => {}
            CrumblePegState::Crumbling => {
                if self.timer >= self.crumble_delay {
                    self.state = CrumblePegState::Crumbled;
                    self.timer = 0.0;
                }
            }
            CrumblePegState::Crumbled => {
                if self.respawn_delay > 0.0 && self.timer >= self.respawn_delay {
                    self.state = CrumblePegState::Respawning;
                    self.timer = 0.0;
                }
            }
            CrumblePegState::Respawning => {
                // Quick respawn transition
                if self.timer >= 0.5 {
                    self.state = CrumblePegState::Idle;
                    self.timer = 0.0;
                }
            }
        }
    }

    /// Whether the platform is currently solid (can be stood on).
    pub fn is_solid(&self) -> bool {
        matches!(self.state, CrumblePegState::Idle | CrumblePegState::Crumbling)
    }
}

impl Default for CrumblingPegComponent {
    fn default() -> Self {
        Self {
            state: CrumblePegState::Idle,
            crumble_delay: 0.8,
            respawn_delay: 3.0,
            timer: 0.0,
            shake_intensity: 2.0,
        }
    }
}

impl Component for CrumblingPegComponent {}
