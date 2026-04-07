//! Render system: draws all visible entities sorted by z-order.

use openclaw_core::ecs::{System, World};

use crate::components::render::RenderComponent;
use crate::components::transform::TransformComponent;

/// Collects and sorts renderable entities for drawing.
/// Actual rendering is performed by the game view using the renderer trait.
pub struct RenderSystem;

impl System for RenderSystem {
    fn name(&self) -> &str {
        "RenderSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        // Collect visible entities sorted by z-order.
        // The actual draw calls are made by ClawHumanView; this system
        // ensures render components are in a consistent state.
        let entities: Vec<_> = world.entities();

        for entity in entities {
            let visible = world
                .get_component::<RenderComponent>(entity)
                .map_or(false, |r| r.visible);

            if !visible {
                continue;
            }

            // Ensure entities with render components also have transforms.
            if !world.has_component::<TransformComponent>(entity) {
                continue;
            }

            // Sync flip_x based on kinematic velocity if applicable.
            let vel_x = world
                .get_component::<crate::components::kinematic::KinematicComponent>(entity)
                .map(|k| k.velocity.x);

            if let Some(vx) = vel_x {
                if vx.abs() > 0.1 {
                    if let Some(render) = world.get_component_mut::<RenderComponent>(entity) {
                        render.flip_x = vx < 0.0;
                    }
                }
            }
        }
    }
}
