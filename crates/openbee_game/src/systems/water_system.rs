//! Water system: detects entity overlap with water zones and applies aquatic
//! physics, visual effects, and breath management.

use glam::Vec2;
use openbee_core::ecs::{Entity, System, World};

use crate::components::hazards::water_zone::WaterZoneComponent;
use crate::components::health::HealthComponent;
use crate::components::kinematic::KinematicComponent;
use crate::components::movement::advanced::SwimmingComponent;
use crate::components::transform::TransformComponent;

/// Handles water-zone interactions for all entities.
pub struct WaterSystem;

impl System for WaterSystem {
    fn name(&self) -> &str {
        "WaterSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        // Collect water zones first (zone entity id + cloned data).
        let zones: Vec<(Entity, WaterZoneComponent)> = entities
            .iter()
            .filter_map(|&e| {
                world
                    .get_component::<WaterZoneComponent>(e)
                    .cloned()
                    .map(|wz| (e, wz))
            })
            .collect();

        // Update wave timers on each zone.
        for &(zone_entity, _) in &zones {
            if let Some(wz) = world.get_component_mut::<WaterZoneComponent>(zone_entity) {
                wz.wave_timer += dt;
            }
        }

        // Process every entity that can swim against every water zone.
        for &entity in &entities {
            let entity_pos = match world.get_component::<TransformComponent>(entity) {
                Some(t) => t.position,
                None => continue,
            };

            let has_swimming = world.has_component::<SwimmingComponent>(entity);
            if !has_swimming {
                continue;
            }

            let mut in_any_zone = false;
            let mut on_surface = false;
            let mut accumulated_current = Vec2::ZERO;
            let mut zone_drag: f32 = 1.0;
            let mut zone_buoyancy: f32 = 0.0;

            for (_zone_entity, zone) in &zones {
                let (zx, zy, zw, zh) = zone.zone_rect;
                let in_zone = entity_pos.x >= zx
                    && entity_pos.x <= zx + zw
                    && entity_pos.y >= zy
                    && entity_pos.y <= zy + zh;

                if !in_zone {
                    continue;
                }

                in_any_zone = true;

                // Consider entity "on surface" when near the surface Y.
                let surface_threshold = 8.0;
                let wave_offset =
                    zone.wave_amplitude * (zone.wave_timer * zone.wave_frequency).sin();
                let effective_surface = zone.surface_y + wave_offset;

                if (entity_pos.y - effective_surface).abs() < surface_threshold {
                    on_surface = true;
                }

                // Accumulate zone effects.
                accumulated_current += Vec2::new(
                    zone.current_direction.0 * zone.current_strength,
                    zone.current_direction.1 * zone.current_strength,
                );
                zone_drag = zone_drag.min(zone.drag);
                zone_buoyancy += zone.buoyancy;
            }

            // Write swimming state.
            let was_submerged = world
                .get_component::<SwimmingComponent>(entity)
                .is_some_and(|s| s.is_submerged);

            if let Some(swim) = world.get_component_mut::<SwimmingComponent>(entity) {
                let entering = in_any_zone && !was_submerged;
                let exiting = !in_any_zone && was_submerged;

                swim.is_submerged = in_any_zone;
                swim.is_on_surface = on_surface;

                // Deplete / recover breath.
                if swim.is_submerged && !swim.is_on_surface {
                    swim.breath_remaining = (swim.breath_remaining - dt).max(0.0);
                } else if swim.breath_remaining < swim.breath_max {
                    swim.breath_remaining = (swim.breath_remaining + dt * 3.0).min(swim.breath_max);
                }

                // Splash effects on transition.
                if entering || exiting {
                    // In a full implementation we would spawn a particle effect
                    // using swim.splash_effect or similar.
                    let _ = entering;
                }
            }

            // Apply aquatic physics.
            if in_any_zone {
                if let Some(kin) = world.get_component_mut::<KinematicComponent>(entity) {
                    // Water drag – scale velocity.
                    kin.velocity *= zone_drag;

                    // Buoyancy – upward force.
                    kin.acceleration.y -= zone_buoyancy;

                    // Water current.
                    kin.velocity += accumulated_current * dt;
                }

                // Drowning damage.
                let breath = world
                    .get_component::<SwimmingComponent>(entity)
                    .map_or(1.0, |s| s.breath_remaining);
                let drown_rate = world
                    .get_component::<SwimmingComponent>(entity)
                    .map_or(0.0, |s| s.drown_damage_rate);

                if breath <= 0.0 {
                    if let Some(health) = world.get_component_mut::<HealthComponent>(entity) {
                        let dmg = (drown_rate * dt).ceil() as i32;
                        health.apply_damage(dmg);
                    }
                }
            }
        }
    }
}
