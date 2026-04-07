//! Hazard system: updates all environmental hazards.

use openbee_core::ecs::{System, World};

use crate::components::hazards::floor_spike::{FloorSpikeComponent, SpikeState};
use crate::components::hazards::saw_blade::SawBladeComponent;
use crate::components::hazards::spring_board::SpringBoardComponent;
use crate::components::hazards::stepping_ground::SteppingGroundComponent;
use crate::components::hazards::area_damage::AreaDamageComponent;

/// Updates environmental hazards: spikes, saw blades, spring boards, etc.
pub struct HazardSystem;

impl System for HazardSystem {
    fn name(&self) -> &str {
        "HazardSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        for &entity in &entities {
            // Floor spikes
            if let Some(spike) = world.get_component_mut::<FloorSpikeComponent>(entity) {
                if spike.active {
                    spike.timer += dt;
                    let half_cycle = spike.cycle_time * 0.5;
                    let quarter_cycle = spike.cycle_time * 0.25;

                    spike.state = if spike.timer < quarter_cycle {
                        SpikeState::Retracted
                    } else if spike.timer < half_cycle {
                        SpikeState::Extending
                    } else if spike.timer < half_cycle + quarter_cycle {
                        SpikeState::Extended
                    } else {
                        SpikeState::Retracting
                    };

                    if spike.timer >= spike.cycle_time {
                        spike.timer -= spike.cycle_time;
                    }
                }
            }

            // Saw blades
            if let Some(saw) = world.get_component_mut::<SawBladeComponent>(entity) {
                saw.current_angle += saw.rotation_speed * dt;
                if saw.current_angle > std::f32::consts::TAU {
                    saw.current_angle -= std::f32::consts::TAU;
                }
            }

            // Spring boards
            if let Some(spring) = world.get_component_mut::<SpringBoardComponent>(entity) {
                if spring.timer > 0.0 {
                    spring.timer = (spring.timer - dt).max(0.0);
                }
            }

            // Stepping ground
            if let Some(step) = world.get_component_mut::<SteppingGroundComponent>(entity) {
                if step.falling {
                    // Already handled by physics
                } else if step.timer > 0.0 {
                    step.timer += dt;
                    if step.timer >= step.fall_delay {
                        step.falling = true;
                    }
                }
            }

            // Area damage
            if let Some(area) = world.get_component_mut::<AreaDamageComponent>(entity) {
                area.timer += dt;
                if area.timer >= area.interval {
                    area.timer -= area.interval;
                    // Damage application uses collision detection from combat system.
                }
            }
        }
    }
}
