//! Render component for sprite display.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Controls how an entity is drawn on screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderComponent {
    /// Identifier for the sprite or image set to draw.
    pub sprite_id: Option<String>,
    /// Whether the entity is currently visible.
    pub visible: bool,
    /// Flip the sprite horizontally (for facing direction).
    pub flip_x: bool,
    /// Draw order; lower values are drawn first (behind higher values).
    pub z_order: i32,
    /// RGBA color modulation applied to the sprite.
    pub color_mod: [u8; 4],
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            sprite_id: None,
            visible: true,
            flip_x: false,
            z_order: 0,
            color_mod: [255, 255, 255, 255],
        }
    }
}

impl Component for RenderComponent {}
