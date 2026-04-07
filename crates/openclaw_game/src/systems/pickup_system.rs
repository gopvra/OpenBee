//! Pickup system: handles item collection by the player.

use openclaw_core::ecs::{System, World};
use openclaw_core::render::Rect;

use crate::components::ammo::AmmoComponent;
use crate::components::collision::CollisionComponent;
use crate::components::health::HealthComponent;
use crate::components::life::LifeComponent;
use crate::components::pickup::{AmmoType, PickupComponent, PickupType};
use crate::components::score::ScoreComponent;
use crate::components::transform::TransformComponent;

/// Detects overlap between the player and pickup items, applying their effects.
pub struct PickupSystem;

impl System for PickupSystem {
    fn name(&self) -> &str {
        "PickupSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<_> = world.entities();

        // Find player entities (those with ScoreComponent - only the player has one)
        let mut players: Vec<(openclaw_core::ecs::Entity, glam::Vec2, Rect)> = Vec::new();
        for &entity in &entities {
            if let (Some(transform), Some(collision), Some(_score)) = (
                world.get_component::<TransformComponent>(entity),
                world.get_component::<CollisionComponent>(entity),
                world.get_component::<ScoreComponent>(entity),
            ) {
                players.push((entity, transform.position, collision.hit_rect));
            }
        }

        // Collect pickups to process
        let mut pickups_collected: Vec<(openclaw_core::ecs::Entity, openclaw_core::ecs::Entity, PickupType, i32)> = Vec::new();

        for &entity in &entities {
            if let (Some(transform), Some(collision), Some(pickup)) = (
                world.get_component::<TransformComponent>(entity),
                world.get_component::<CollisionComponent>(entity),
                world.get_component::<PickupComponent>(entity),
            ) {
                for &(player_entity, ref player_pos, ref player_rect) in &players {
                    let pickup_world = Rect::new(
                        transform.position.x + collision.hit_rect.x,
                        transform.position.y + collision.hit_rect.y,
                        collision.hit_rect.width,
                        collision.hit_rect.height,
                    );
                    let player_world = Rect::new(
                        player_pos.x + player_rect.x,
                        player_pos.y + player_rect.y,
                        player_rect.width,
                        player_rect.height,
                    );
                    if pickup_world.overlaps(&player_world) {
                        pickups_collected.push((
                            entity,
                            player_entity,
                            pickup.pickup_type,
                            pickup.value,
                        ));
                    }
                }
            }
        }

        // Apply pickup effects
        for (pickup_entity, player_entity, pickup_type, value) in pickups_collected {
            match pickup_type {
                PickupType::Treasure(treasure_type) => {
                    let base_points = treasure_type.base_value();
                    if let Some(score) = world.get_component_mut::<ScoreComponent>(player_entity) {
                        score.add(base_points);
                    }
                }
                PickupType::Health => {
                    if let Some(health) = world.get_component_mut::<HealthComponent>(player_entity) {
                        health.heal(value);
                    }
                }
                PickupType::Ammo(ammo_type) => {
                    if let Some(ammo) = world.get_component_mut::<AmmoComponent>(player_entity) {
                        match ammo_type {
                            AmmoType::Pistol => ammo.add_pistol(value),
                            AmmoType::Dynamite => ammo.add_dynamite(value),
                            AmmoType::Magic => ammo.add_magic(value),
                        }
                    }
                }
                PickupType::ExtraLife => {
                    if let Some(life) = world.get_component_mut::<LifeComponent>(player_entity) {
                        life.add_life();
                    }
                }
                PickupType::Powerup(_) | PickupType::MapPiece | PickupType::Gem => {
                    // Handled by powerup system or level manager respectively.
                    if let Some(score) = world.get_component_mut::<ScoreComponent>(player_entity) {
                        score.add(value as u64);
                    }
                }
            }

            // Remove the picked-up entity
            world.destroy_entity(pickup_entity);
        }
    }
}
