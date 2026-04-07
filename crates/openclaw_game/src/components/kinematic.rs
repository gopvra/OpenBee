//! Kinematic movement component for velocity-based motion.

use glam::Vec2;
use openclaw_core::ecs::Component;
/// Velocity and acceleration state for entities that move.
#[derive(Debug, Clone)]
pub struct KinematicComponent {
    /// Current velocity in pixels per second.
    pub velocity: Vec2,
    /// Current acceleration in pixels per second squared.
    pub acceleration: Vec2,
    /// Maximum speed in each axis (pixels per second).
    pub max_speed: Vec2,
    /// Whether the entity is currently on the ground.
    pub on_ground: bool,
}

impl Default for KinematicComponent {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            max_speed: Vec2::new(400.0, 800.0),
            on_ground: false,
        }
    }
}

impl Component for KinematicComponent {}
