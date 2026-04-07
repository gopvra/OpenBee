//! Combat system: handles attacks, damage, and hit detection.

use openclaw_core::ecs::{Entity, System, World};
use openclaw_core::render::Rect;

use crate::components::collision::CollisionComponent;
use crate::components::destroyable::DestroyableComponent;
use crate::components::health::HealthComponent;
use crate::components::transform::TransformComponent;

/// Processes combat interactions: melee attacks, damage application, and death.
pub struct CombatSystem;

impl CombatSystem {
    /// Check if two world-space rectangles overlap, given entity positions and local rects.
    fn rects_overlap(
        pos_a: &glam::Vec2,
        rect_a: &Rect,
        pos_b: &glam::Vec2,
        rect_b: &Rect,
    ) -> bool {
        let a = Rect::new(
            pos_a.x + rect_a.x,
            pos_a.y + rect_a.y,
            rect_a.width,
            rect_a.height,
        );
        let b = Rect::new(
            pos_b.x + rect_b.x,
            pos_b.y + rect_b.y,
            rect_b.width,
            rect_b.height,
        );
        a.overlaps(&b)
    }
}

impl System for CombatSystem {
    fn name(&self) -> &str {
        "CombatSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        // Update invulnerability timers
        for &entity in &entities {
            if let Some(health) = world.get_component_mut::<HealthComponent>(entity) {
                if health.invulnerable && health.invulnerability_timer > 0.0 {
                    health.invulnerability_timer -= dt;
                    if health.invulnerability_timer <= 0.0 {
                        health.invulnerable = false;
                        health.invulnerability_timer = 0.0;
                    }
                }
            }
        }

        // Collect attackers (entities with non-zero attack rects)
        let mut attackers: Vec<(Entity, glam::Vec2, Rect, u32)> = Vec::new();
        for &entity in &entities {
            if let (Some(transform), Some(collision)) = (
                world.get_component::<TransformComponent>(entity),
                world.get_component::<CollisionComponent>(entity),
            ) {
                if collision.attack_rect.width > 0.0 && collision.attack_rect.height > 0.0 {
                    attackers.push((
                        entity,
                        transform.position,
                        collision.attack_rect,
                        collision.collision_mask,
                    ));
                }
            }
        }

        // Check attacks against all damageable entities
        let mut damage_events: Vec<(Entity, i32)> = Vec::new();
        for &entity in &entities {
            if let (Some(transform), Some(collision), Some(_health)) = (
                world.get_component::<TransformComponent>(entity),
                world.get_component::<CollisionComponent>(entity),
                world.get_component::<HealthComponent>(entity),
            ) {
                for &(attacker, ref atk_pos, ref atk_rect, atk_mask) in &attackers {
                    if attacker == entity {
                        continue;
                    }
                    // Check layer/mask compatibility
                    if atk_mask & collision.collision_layer == 0 {
                        continue;
                    }
                    if Self::rects_overlap(atk_pos, atk_rect, &transform.position, &collision.hit_rect)
                    {
                        // Base damage of 10 - in a full implementation this comes from the weapon.
                        damage_events.push((entity, 10));
                    }
                }
            }
        }

        // Apply damage
        for (entity, damage) in damage_events {
            if let Some(health) = world.get_component_mut::<HealthComponent>(entity) {
                let dealt = health.apply_damage(damage);
                if dealt > 0 {
                    // Grant brief invulnerability
                    health.invulnerable = true;
                    health.invulnerability_timer = 0.5;
                }

                if health.is_dead() {
                    if let Some(destroyable) = world.get_component_mut::<DestroyableComponent>(entity) {
                        destroyable.is_destroyed = true;
                    }
                }
            }
        }
    }
}
