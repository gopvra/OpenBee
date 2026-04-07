//! Transform component for position, rotation, and scale.

use glam::Vec2;
use openclaw_core::ecs::Component;
/// Spatial transformation of an entity in the game world.
#[derive(Debug, Clone)]
pub struct TransformComponent {
    /// World-space position in pixels.
    pub position: Vec2,
    /// Rotation in radians.
    pub rotation: f32,
    /// Non-uniform scale factor.
    pub scale: Vec2,
}

impl TransformComponent {
    /// Create a new transform at the given position with no rotation and unit scale.
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Component for TransformComponent {}
