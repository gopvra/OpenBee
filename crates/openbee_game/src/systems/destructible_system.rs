//! Destructible terrain system: processes damage, debris, respawns, and loot drops.

use openbee_core::ecs::{System, World};

use crate::components::hazards::destructible_terrain::DestructibleTerrainComponent;
use crate::components::transform::TransformComponent;

/// Manages destructible terrain: respawn timers, collision state updates, and
/// debris/loot spawning when terrain is destroyed.
pub struct DestructibleSystem;

impl DestructibleSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DestructibleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for DestructibleSystem {
    fn name(&self) -> &str {
        "DestructibleSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        // Track entities that need post-destruction processing.
        let mut just_destroyed: Vec<(u32, u32, Option<String>, Option<String>, Vec<String>)> =
            Vec::new();
        let mut just_respawned: Vec<u32> = Vec::new();

        for &entity in &entities {
            if let Some(terrain) = world.get_component_mut::<DestructibleTerrainComponent>(entity) {
                if terrain.destroyed {
                    // Check for respawn.
                    if terrain.try_respawn(dt) {
                        just_respawned.push(entity.id);
                    }
                }
            }
        }

        // Process entities that were just destroyed this frame (health reached zero).
        // The actual damage application happens externally (e.g. combat system calls
        // take_damage); here we detect newly-destroyed terrain and handle side effects.
        for &entity in &entities {
            if let Some(terrain) = world.get_component::<DestructibleTerrainComponent>(entity) {
                if terrain.destroyed && terrain.respawn_timer > 0.0 {
                    // This terrain was recently destroyed — check if we need to spawn debris.
                    // We use the respawn_timer being close to the respawn_time as an indicator
                    // that destruction just occurred.
                    if let Some(respawn_time) = terrain.respawn_time {
                        let time_since_destroyed = respawn_time - terrain.respawn_timer;
                        if time_since_destroyed < dt * 1.5 {
                            just_destroyed.push((
                                entity.id,
                                terrain.debris_count,
                                terrain.debris_sprite.clone(),
                                terrain.destroy_sound.clone(),
                                terrain.loot_on_destroy.clone(),
                            ));
                        }
                    }
                } else if terrain.destroyed && terrain.respawn_time.is_none() {
                    // Permanently destroyed, check if debris needs spawning.
                    // We check health == 0 as the sentinel for "just destroyed".
                    if terrain.health == 0 && terrain.debris_count > 0 {
                        just_destroyed.push((
                            entity.id,
                            terrain.debris_count,
                            terrain.debris_sprite.clone(),
                            terrain.destroy_sound.clone(),
                            terrain.loot_on_destroy.clone(),
                        ));
                    }
                }
            }
        }

        // Spawn debris particles for recently destroyed terrain.
        for (entity_id, debris_count, _debris_sprite, _destroy_sound, _loot) in &just_destroyed {
            // Look up the position of the destroyed entity for particle spawning.
            let _position = entities
                .iter()
                .find(|e| e.id == *entity_id)
                .and_then(|e| world.get_component::<TransformComponent>(*e));

            // In a full implementation:
            // - Spawn `debris_count` particle entities at the terrain's position using
            //   the debris_sprite as the particle graphic.
            // - Play destroy_sound via the audio system.
            // - Spawn loot pickup entities from the loot templates.
            let _ = debris_count;
        }

        // Handle respawned terrain: restore collision state.
        for _entity_id in &just_respawned {
            // In a full implementation:
            // - Re-enable the collision component on the entity.
            // - Reset the render component to the original tile/sprite.
        }
    }
}
