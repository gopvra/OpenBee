//! Toggle peg AI: a platform that alternates between solid and intangible.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// State of a toggle peg platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TogglePegState {
    /// Currently solid.
    Solid,
    /// Transitioning to intangible (fading out).
    FadingOut,
    /// Currently intangible (not solid).
    Intangible,
    /// Transitioning to solid (fading in).
    FadingIn,
}

/// A platform that toggles between solid and intangible on a timer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TogglePegComponent {
    /// Current state.
    pub state: TogglePegState,
    /// Duration the peg stays solid (seconds).
    pub on_time: f32,
    /// Duration the peg stays intangible (seconds).
    pub off_time: f32,
    /// Transition fade duration (seconds).
    pub fade_time: f32,
    /// Current timer within the current phase.
    pub timer: f32,
    /// Phase offset for synchronizing groups (seconds).
    pub phase_offset: f32,
    /// Toggle group ID (pegs with the same group toggle in sync).
    pub group_id: u32,
}

impl TogglePegComponent {
    /// Update the toggle state machine.
    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        match self.state {
            TogglePegState::Solid => {
                if self.timer >= self.on_time {
                    self.state = TogglePegState::FadingOut;
                    self.timer = 0.0;
                }
            }
            TogglePegState::FadingOut => {
                if self.timer >= self.fade_time {
                    self.state = TogglePegState::Intangible;
                    self.timer = 0.0;
                }
            }
            TogglePegState::Intangible => {
                if self.timer >= self.off_time {
                    self.state = TogglePegState::FadingIn;
                    self.timer = 0.0;
                }
            }
            TogglePegState::FadingIn => {
                if self.timer >= self.fade_time {
                    self.state = TogglePegState::Solid;
                    self.timer = 0.0;
                }
            }
        }
    }

    /// Whether the peg is currently solid (can be stood on).
    pub fn is_solid(&self) -> bool {
        matches!(
            self.state,
            TogglePegState::Solid | TogglePegState::FadingOut
        )
    }

    /// Visual opacity for rendering (0.0 = invisible, 1.0 = fully visible).
    pub fn opacity(&self) -> f32 {
        match self.state {
            TogglePegState::Solid => 1.0,
            TogglePegState::FadingOut => {
                if self.fade_time > 0.0 {
                    1.0 - (self.timer / self.fade_time)
                } else {
                    0.0
                }
            }
            TogglePegState::Intangible => 0.3,
            TogglePegState::FadingIn => {
                if self.fade_time > 0.0 {
                    0.3 + 0.7 * (self.timer / self.fade_time)
                } else {
                    1.0
                }
            }
        }
    }
}

impl Default for TogglePegComponent {
    fn default() -> Self {
        Self {
            state: TogglePegState::Solid,
            on_time: 2.0,
            off_time: 2.0,
            fade_time: 0.3,
            timer: 0.0,
            phase_offset: 0.0,
            group_id: 0,
        }
    }
}

impl Component for TogglePegComponent {}
