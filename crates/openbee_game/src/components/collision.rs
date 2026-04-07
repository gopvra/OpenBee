//! Collision component defining hit, attack, and clip rectangles.

use openbee_core::ecs::Component;
use openbee_core::render::Rect;
/// Collision geometry and filtering for an entity.
#[derive(Debug, Clone)]
pub struct CollisionComponent {
    /// Bounding rectangle for receiving damage / being hit.
    pub hit_rect: Rect,
    /// Bounding rectangle for dealing damage when attacking.
    pub attack_rect: Rect,
    /// Clipping rectangle for world geometry collisions.
    pub clip_rect: Rect,
    /// Bitmask for which layers this entity collides with.
    pub collision_mask: u32,
    /// The layer(s) this entity belongs to.
    pub collision_layer: u32,
}

impl Default for CollisionComponent {
    fn default() -> Self {
        Self {
            hit_rect: Rect::new(0.0, 0.0, 32.0, 32.0),
            attack_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            clip_rect: Rect::new(0.0, 0.0, 32.0, 32.0),
            collision_mask: 0xFFFFFFFF,
            collision_layer: 1,
        }
    }
}

impl Component for CollisionComponent {}
