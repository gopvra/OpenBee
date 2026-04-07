//! Physics body component.

use openbee_core::ecs::Component;
use openbee_core::physics::BodyHandle;
use serde::{Deserialize, Serialize};

/// Type of physics body for simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsBodyType {
    /// Fully simulated, affected by gravity and forces.
    Dynamic,
    /// Moves only via explicit position/velocity; not affected by forces.
    Kinematic,
    /// Never moves; used for terrain and walls.
    Static,
}

/// Links an entity to a physics body in the physics world.
#[derive(Debug, Clone)]
pub struct PhysicsComponent {
    /// Handle to the body in the physics world (None until created).
    pub body_handle: Option<BodyHandle>,
    /// The type of physics body.
    pub body_type: PhysicsBodyType,
    /// Gravity scale (1.0 = normal, 0.0 = no gravity).
    pub gravity_scale: f32,
}

impl Default for PhysicsComponent {
    fn default() -> Self {
        Self {
            body_handle: None,
            body_type: PhysicsBodyType::Dynamic,
            gravity_scale: 1.0,
        }
    }
}

impl Component for PhysicsComponent {}
