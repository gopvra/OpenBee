//! Spring board component for launch pads.

use glam::Vec2;
use openbee_core::ecs::Component;
/// A springboard that launches entities that land on it.
#[derive(Debug, Clone)]
pub struct SpringBoardComponent {
    /// Force applied when launched (pixels per second).
    pub launch_force: f32,
    /// Normalized direction of launch.
    pub direction: Vec2,
    /// Cooldown between launches in seconds.
    pub cooldown: f32,
    /// Current cooldown timer.
    pub timer: f32,
}

impl Default for SpringBoardComponent {
    fn default() -> Self {
        Self {
            launch_force: 800.0,
            direction: Vec2::NEG_Y,
            cooldown: 0.5,
            timer: 0.0,
        }
    }
}

impl Component for SpringBoardComponent {}
