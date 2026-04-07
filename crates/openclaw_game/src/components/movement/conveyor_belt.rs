//! Conveyor belt component that pushes entities in a direction.

use glam::Vec2;
use openclaw_core::ecs::Component;
/// A surface that applies a constant velocity to entities standing on it.
#[derive(Debug, Clone)]
pub struct ConveyorBeltComponent {
    /// Speed of the belt in pixels per second.
    pub speed: f32,
    /// Normalized direction of movement.
    pub direction: Vec2,
}

impl Default for ConveyorBeltComponent {
    fn default() -> Self {
        Self {
            speed: 100.0,
            direction: Vec2::X,
        }
    }
}

impl Component for ConveyorBeltComponent {}
