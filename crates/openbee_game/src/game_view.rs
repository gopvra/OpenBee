//! Player view handling rendering pipeline (BeeHumanView equivalent).

use glam::Vec2;
use openbee_core::ecs::{Entity, World};
use openbee_core::render::renderer::Camera;
use openbee_core::render::{Color, Rect, Renderer};

use crate::components::render::RenderComponent;
use crate::components::transform::TransformComponent;

/// The human player's view of the game world, managing camera and rendering.
pub struct BeeHumanView {
    pub camera: Camera,
    pub player_entity: Option<Entity>,
    pub screen_width: u32,
    pub screen_height: u32,
    /// Camera smoothing factor (0 = instant, 1 = no movement).
    pub camera_lerp: f32,
    /// Camera bounds for the current level.
    pub camera_bounds: Option<Rect>,
}

impl BeeHumanView {
    /// Create a new human view with the given screen dimensions.
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            camera: Camera {
                position: Vec2::ZERO,
                zoom: 1.0,
                viewport: Rect::new(0.0, 0.0, screen_width as f32, screen_height as f32),
            },
            player_entity: None,
            screen_width,
            screen_height,
            camera_lerp: 0.1,
            camera_bounds: None,
        }
    }

    /// Update the camera to follow the player entity.
    pub fn update(&mut self, world: &World, _dt: f32) {
        if let Some(player) = self.player_entity {
            if let Some(transform) = world.get_component::<TransformComponent>(player) {
                let target = transform.position;
                // Smooth camera follow
                self.camera.position = self.camera.position.lerp(target, self.camera_lerp);
                // Clamp to level bounds
                if let Some(bounds) = &self.camera_bounds {
                    let half_w = self.screen_width as f32 * 0.5 / self.camera.zoom;
                    let half_h = self.screen_height as f32 * 0.5 / self.camera.zoom;
                    self.camera.position.x = self.camera.position.x
                        .max(bounds.x + half_w)
                        .min(bounds.x + bounds.width - half_w);
                    self.camera.position.y = self.camera.position.y
                        .max(bounds.y + half_h)
                        .min(bounds.y + bounds.height - half_h);
                }
            }
        }
    }

    /// Render all visible entities through the given renderer.
    pub fn render(&self, world: &World, renderer: &mut dyn Renderer) {
        renderer.set_camera(&self.camera);
        renderer.clear(Color::BLACK);

        // Gather all renderable entities and sort by z-order
        let mut renderables: Vec<(Entity, i32)> = Vec::new();
        for entity in world.entities() {
            if let Some(render_comp) = world.get_component::<RenderComponent>(entity) {
                if render_comp.visible {
                    renderables.push((entity, render_comp.z_order));
                }
            }
        }
        renderables.sort_by_key(|&(_, z)| z);

        for (entity, _) in &renderables {
            if let (Some(_transform), Some(_render_comp)) = (
                world.get_component::<TransformComponent>(*entity),
                world.get_component::<RenderComponent>(*entity),
            ) {
                // Sprite rendering is delegated to the render system.
                // This view manages the camera and rendering pipeline order.
            }
        }

        renderer.present();
    }

    /// Set the camera bounds for the current level.
    pub fn set_level_bounds(&mut self, bounds: Rect) {
        self.camera_bounds = Some(bounds);
    }

    /// Attach the camera to follow a player entity.
    pub fn set_player(&mut self, entity: Entity) {
        self.player_entity = Some(entity);
    }

    /// Convert a screen position to world coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let half_w = self.screen_width as f32 * 0.5;
        let half_h = self.screen_height as f32 * 0.5;
        Vec2::new(
            (screen_pos.x - half_w) / self.camera.zoom + self.camera.position.x,
            (screen_pos.y - half_h) / self.camera.zoom + self.camera.position.y,
        )
    }

    /// Convert a world position to screen coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let half_w = self.screen_width as f32 * 0.5;
        let half_h = self.screen_height as f32 * 0.5;
        Vec2::new(
            (world_pos.x - self.camera.position.x) * self.camera.zoom + half_w,
            (world_pos.y - self.camera.position.y) * self.camera.zoom + half_h,
        )
    }
}
