//! Followable component for entities that track a target.

use glam::Vec2;
use openbee_core::ecs::{Component, Entity};
/// An entity that follows a target entity with configurable tracking behavior.
#[derive(Debug, Clone)]
pub struct FollowableComponent {
    /// The entity to follow.
    pub target: Option<Entity>,
    /// Offset from the target position.
    pub offset: Vec2,
    /// Follow speed in pixels per second.
    pub speed: f32,
    /// Maximum distance before snapping to target.
    pub max_distance: f32,
}

impl Default for FollowableComponent {
    fn default() -> Self {
        Self {
            target: None,
            offset: Vec2::ZERO,
            speed: 200.0,
            max_distance: 500.0,
        }
    }
}

impl Component for FollowableComponent {}
