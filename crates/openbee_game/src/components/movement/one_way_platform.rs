//! One-way platform component that allows passage from one side only.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A platform that only blocks movement from one direction, allowing entities to
/// pass through from the other side. Supports a drop-through mechanic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneWayPlatformComponent {
    /// The direction from which entities can pass through.
    pub direction: OneWayDirection,
    /// Thickness of the collision region in pixels.
    pub thickness: f32,
    /// Whether the player can press down to drop through.
    pub allow_drop_through: bool,
    /// Remaining time the platform is in drop-through state (seconds).
    pub drop_through_timer: f32,
    /// Whether the entity is currently dropping through this platform.
    pub is_dropping: bool,
}

/// Direction from which entities are allowed to pass through the platform.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OneWayDirection {
    /// Can pass from below (solid when standing on top).
    Up,
    /// Can pass from above (solid when hitting from below).
    Down,
    /// Can pass from the left.
    Left,
    /// Can pass from the right.
    Right,
}

impl Default for OneWayPlatformComponent {
    fn default() -> Self {
        Self {
            direction: OneWayDirection::Up,
            thickness: 16.0,
            allow_drop_through: true,
            drop_through_timer: 0.0,
            is_dropping: false,
        }
    }
}

impl Component for OneWayPlatformComponent {}
