//! Checkpoint system: manages checkpoint activation and respawning.

use openbee_core::ecs::{System, World};

use crate::components::checkpoint::CheckpointComponent;
use crate::components::score::ScoreComponent;
use crate::components::transform::TransformComponent;

/// Activates checkpoints when the player overlaps them.
pub struct CheckpointSystem;

impl System for CheckpointSystem {
    fn name(&self) -> &str {
        "CheckpointSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<_> = world.entities();

        // Find player position
        let mut player_pos = None;
        for &entity in &entities {
            if world.has_component::<ScoreComponent>(entity) {
                if let Some(transform) = world.get_component::<TransformComponent>(entity) {
                    player_pos = Some(transform.position);
                }
            }
        }

        let player_pos = match player_pos {
            Some(p) => p,
            None => return,
        };

        // Check all checkpoints for activation
        for &entity in &entities {
            let should_activate = {
                let checkpoint = world.get_component::<CheckpointComponent>(entity);
                let transform = world.get_component::<TransformComponent>(entity);
                match (checkpoint, transform) {
                    (Some(cp), Some(t)) if !cp.activated => {
                        let dist = (player_pos - t.position).length();
                        dist < 48.0 // activation radius
                    }
                    _ => false,
                }
            };

            if should_activate {
                if let Some(cp) = world.get_component_mut::<CheckpointComponent>(entity) {
                    cp.activated = true;
                    tracing::info!(
                        "Checkpoint {} activated at ({}, {})",
                        cp.checkpoint_id,
                        cp.respawn_x,
                        cp.respawn_y
                    );
                }
            }
        }
    }
}
