//! Physics module wrapping Rapier2D for 2D rigid body simulation.

pub mod body;
pub mod collision;
pub mod debug_draw;
pub mod world;

pub use body::{BodyDesc, BodyHandle, BodyShape};
pub use collision::{CollisionEvent, CollisionType, ContactListener};
pub use debug_draw::PhysicsDebugDrawer;
pub use world::PhysicsWorld;

/// Conversion factor between pixel coordinates and physics meters.
pub const PIXELS_PER_METER: f32 = 75.0;
