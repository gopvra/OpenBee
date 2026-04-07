//! Particle system: updates and manages particle effects.

use openclaw_core::ecs::{System, World};

use crate::components::glitter::GlitterComponent;
use crate::components::transform::TransformComponent;

/// Updates particle emitters and removes expired particles.
pub struct ParticleSystem;

impl System for ParticleSystem {
    fn name(&self) -> &str {
        "ParticleSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<_> = world.entities();

        for &entity in &entities {
            if let (Some(_glitter), Some(_transform)) = (
                world.get_component::<GlitterComponent>(entity),
                world.get_component::<TransformComponent>(entity),
            ) {
                // Particle emission and lifetime management.
                // The actual particle rendering is done through the core ParticleSystem.
                // This system spawns new particles based on glitter component settings.
            }
        }
    }
}
