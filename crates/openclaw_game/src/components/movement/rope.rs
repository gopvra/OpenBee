//! Rope component for swinging mechanics.

use openclaw_core::ecs::{Component, Entity};
/// A rope that the player or enemies can grab and swing on.
#[derive(Debug, Clone)]
pub struct RopeComponent {
    /// Length of the rope in pixels.
    pub length: f32,
    /// Angular speed of the swing.
    pub swing_speed: f32,
    /// Entity currently attached to the rope, if any.
    pub attached_entity: Option<Entity>,
    /// Current swing angle in radians.
    pub current_angle: f32,
    /// Current angular velocity.
    pub angular_velocity: f32,
}

impl Default for RopeComponent {
    fn default() -> Self {
        Self {
            length: 128.0,
            swing_speed: 2.0,
            attached_entity: None,
            current_angle: 0.0,
            angular_velocity: 0.0,
        }
    }
}

impl Component for RopeComponent {}
