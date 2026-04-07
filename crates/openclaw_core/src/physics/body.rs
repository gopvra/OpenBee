//! Physics body descriptors and handle types.

use glam::Vec2;
use rapier2d::dynamics::RigidBodyHandle;

/// Newtype wrapper around Rapier's rigid body handle for external use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyHandle(pub RigidBodyHandle);

impl BodyHandle {
    /// Access the underlying Rapier handle.
    pub fn raw(&self) -> RigidBodyHandle {
        self.0
    }
}

/// The shape of a physics collider.
#[derive(Debug, Clone)]
pub enum BodyShape {
    /// Circle with a given radius in pixels.
    Circle { radius: f32 },
    /// Axis-aligned rectangle with half-extents in pixels.
    Rectangle { half_width: f32, half_height: f32 },
    /// Capsule defined by a half-height and radius in pixels.
    Capsule { half_height: f32, radius: f32 },
}

/// Describes a physics body to be created, with all relevant properties.
#[derive(Debug, Clone)]
pub struct BodyDesc {
    /// Collider shape.
    pub shape: BodyShape,
    /// Initial position in pixel coordinates.
    pub position: Vec2,
    /// Friction coefficient (0.0 = frictionless, 1.0 = high friction).
    pub friction: f32,
    /// Restitution coefficient (0.0 = no bounce, 1.0 = perfect bounce).
    pub restitution: f32,
    /// Whether the collider is a sensor (detects overlaps but no physical response).
    pub is_sensor: bool,
    /// Optional linear damping.
    pub linear_damping: f32,
    /// Optional gravity scale (1.0 = normal).
    pub gravity_scale: f32,
}

impl Default for BodyDesc {
    fn default() -> Self {
        Self {
            shape: BodyShape::Rectangle {
                half_width: 16.0,
                half_height: 16.0,
            },
            position: Vec2::ZERO,
            friction: 0.3,
            restitution: 0.0,
            is_sensor: false,
            linear_damping: 0.0,
            gravity_scale: 1.0,
        }
    }
}
