//! Spawner system: manages actor and projectile spawning.

use openbee_core::ecs::{System, World};

use crate::components::spawners::actor_spawner::ActorSpawnerComponent;
use crate::components::spawners::projectile_spawner::ProjectileSpawnerComponent;
use crate::components::transform::TransformComponent;

/// Updates all spawner components, creating new entities when conditions are met.
pub struct SpawnerSystem;

impl System for SpawnerSystem {
    fn name(&self) -> &str {
        "SpawnerSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        // Process actor spawners
        for entity in &entities {
            let should_spawn = {
                if let Some(spawner) = world.get_component_mut::<ActorSpawnerComponent>(*entity) {
                    spawner.timer += dt;
                    if spawner.timer >= spawner.spawn_interval {
                        spawner.timer -= spawner.spawn_interval;
                        spawner.spawn_count += 1;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if should_spawn {
                // In a full implementation, this would use ActorTemplateRegistry to spawn.
                // The spawner's transform provides the spawn position.
                if let Some(_transform) = world.get_component::<TransformComponent>(*entity) {
                    // actor_templates::spawn(world, &spawner.template, transform.position);
                }
            }
        }

        // Process projectile spawners
        for entity in &entities {
            if let Some(spawner) = world.get_component_mut::<ProjectileSpawnerComponent>(*entity) {
                if spawner.timer > 0.0 {
                    spawner.timer = (spawner.timer - dt).max(0.0);
                }
            }
        }
    }
}
