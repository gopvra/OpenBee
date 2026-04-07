//! Scene node for rendering an actor's sprite or animation.

use glam::Vec2;

use crate::render::animation::{Animation, AnimationPlayer};
use crate::render::renderer::{Camera, Renderer};
use crate::render::sprite::{Sprite, SpriteSheet};

use super::node::{SceneNode, SceneNodeType};

/// Renders an actor's current sprite/animation frame at its world position.
pub struct ActorSceneNode {
    /// Position in world pixels.
    pub position: Vec2,
    /// Z-order for draw sorting.
    pub z: i32,
    /// Whether this node is visible.
    pub visible: bool,
    /// Whether to flip the sprite horizontally (e.g., facing left).
    pub flip_x: bool,
    /// Whether to flip the sprite vertically.
    pub flip_y: bool,
    /// The current static sprite to render (used if no animation is active).
    pub sprite: Option<Sprite>,
    /// Animation definition.
    pub animation: Option<Animation>,
    /// Animation playback controller.
    pub animation_player: AnimationPlayer,
    /// Sprite sheet containing animation frames.
    sprite_sheet: Option<SpriteSheet>,
}

impl ActorSceneNode {
    /// Create a new actor scene node at the given position.
    pub fn new(position: Vec2, z_order: i32) -> Self {
        Self {
            position,
            z: z_order,
            visible: true,
            flip_x: false,
            flip_y: false,
            sprite: None,
            animation: None,
            animation_player: AnimationPlayer::new(),
            sprite_sheet: None,
        }
    }

    /// Set the static sprite to display.
    pub fn set_sprite(&mut self, sprite: Sprite) {
        self.sprite = Some(sprite);
    }

    /// Set the animation and sprite sheet, and start playing.
    pub fn set_animation(&mut self, animation: Animation, sheet: SpriteSheet) {
        self.animation = Some(animation);
        self.sprite_sheet = Some(sheet);
        self.animation_player.restart();
    }
}

impl SceneNode for ActorSceneNode {
    fn update(&mut self, dt: f32) {
        if let Some(ref anim) = self.animation {
            self.animation_player.update(anim, dt);
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, _camera: &Camera) {
        if !self.visible {
            return;
        }

        // If we have an active animation and sprite sheet, render the current frame.
        if let (Some(ref anim), Some(ref sheet)) = (&self.animation, &self.sprite_sheet) {
            if let Some(sprite_id) = self.animation_player.current_sprite(anim) {
                if let Some(sprite) = sheet.get(sprite_id) {
                    renderer.draw_sprite(sprite, self.position, None, self.flip_x, self.flip_y);
                    return;
                }
            }
        }

        // Fallback to static sprite.
        if let Some(ref sprite) = self.sprite {
            renderer.draw_sprite(sprite, self.position, None, self.flip_x, self.flip_y);
        }
    }

    fn z_order(&self) -> i32 {
        self.z
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn node_type(&self) -> SceneNodeType {
        SceneNodeType::Actor
    }
}
