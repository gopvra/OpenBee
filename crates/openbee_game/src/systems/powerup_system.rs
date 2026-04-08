//! Powerup system: manages active powerups and their timers.

use openbee_core::ecs::{System, World};

use crate::components::health::HealthComponent;
use crate::components::powerup::{PowerupComponent, PowerupType};

/// Ticks active powerup durations and applies/removes their effects.
pub struct PowerupSystem;

impl System for PowerupSystem {
    fn name(&self) -> &str {
        "PowerupSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        for &entity in &entities {
            let expired = {
                let powerup = match world.get_component_mut::<PowerupComponent>(entity) {
                    Some(p) if p.active => p,
                    _ => continue,
                };
                powerup.tick(dt)
            };

            if expired {
                // Remove powerup effects
                let powerup_type = world
                    .get_component::<PowerupComponent>(entity)
                    .map(|p| p.powerup_type);

                if let Some(pt) = powerup_type {
                    match pt {
                        PowerupType::Invincibility => {
                            if let Some(health) = world.get_component_mut::<HealthComponent>(entity) {
                                health.invulnerable = false;
                            }
                        }
                        PowerupType::FireSword
                        | PowerupType::IceSword
                        | PowerupType::LightningSword => {
                            // Remove enhanced weapon state
                        }
                        PowerupType::Catnip => {
                            // Restore normal movement speed
                        }
                        PowerupType::Ghost => {
                            // Restore normal collision
                        }
                    }
                }

                // Remove the powerup component
                world.remove_component::<PowerupComponent>(entity);
            } else {
                // Apply ongoing powerup effects
                let powerup_type = world
                    .get_component::<PowerupComponent>(entity)
                    .map(|p| p.powerup_type);

                if let Some(PowerupType::Invincibility) = powerup_type {
                    if let Some(health) = world.get_component_mut::<HealthComponent>(entity) {
                        health.invulnerable = true;
                    }
                }
            }
        }
    }
}
