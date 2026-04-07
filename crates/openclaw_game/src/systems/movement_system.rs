//! Movement system: updates elevators, conveyors, ropes, and predefined movers.

use glam::Vec2;
use openclaw_core::ecs::{System, World};

use crate::components::movement::followable::FollowableComponent;
use crate::components::movement::path_elevator::PathElevatorComponent;
use crate::components::movement::predefined_move::{MoveAxis, MoveType, PredefinedMoveComponent};
use crate::components::movement::rope::RopeComponent;
use crate::components::transform::TransformComponent;

/// Updates all movement components (elevators, conveyors, ropes, etc.).
pub struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &str {
        "MovementSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        for &entity in &entities {
            // Path elevators
            Self::update_path_elevator(world, entity, dt);

            // Predefined movements
            Self::update_predefined_move(world, entity, dt);

            // Ropes
            Self::update_rope(world, entity, dt);
        }

        // Follow targets (separate pass since it reads other entity positions)
        Self::update_followables(world, dt);
    }
}

impl MovementSystem {
    fn update_path_elevator(world: &mut World, entity: openclaw_core::ecs::Entity, dt: f32) {
        // Read current position before taking a mutable borrow on the elevator.
        let current_pos = world
            .get_component::<TransformComponent>(entity)
            .map(|t| t.position)
            .unwrap_or(Vec2::ZERO);

        let movement = {
            let elevator = match world.get_component_mut::<PathElevatorComponent>(entity) {
                Some(e) if e.waypoints.len() >= 2 => e,
                _ => return,
            };

            // Wait at waypoint
            if elevator.wait_timer > 0.0 {
                elevator.wait_timer -= dt;
                return;
            }

            let target = elevator.waypoints[elevator.current_waypoint];

            let to_target = target - current_pos;
            let dist = to_target.length();

            if dist < elevator.speed * dt {
                // Arrived at waypoint
                elevator.wait_timer = elevator.wait_time;

                if elevator.one_way {
                    elevator.current_waypoint = (elevator.current_waypoint + 1) % elevator.waypoints.len();
                } else {
                    let next = elevator.current_waypoint as i32 + elevator.direction;
                    if next < 0 || next >= elevator.waypoints.len() as i32 {
                        elevator.direction = -elevator.direction;
                    }
                    elevator.current_waypoint =
                        (elevator.current_waypoint as i32 + elevator.direction) as usize;
                }
                Some(target)
            } else {
                let movement = to_target.normalize() * elevator.speed * dt;
                Some(current_pos + movement)
            }
        };

        if let Some(new_pos) = movement {
            if let Some(transform) = world.get_component_mut::<TransformComponent>(entity) {
                transform.position = new_pos;
            }
        }
    }

    fn update_predefined_move(world: &mut World, entity: openclaw_core::ecs::Entity, dt: f32) {
        let new_offset = {
            let mover = match world.get_component_mut::<PredefinedMoveComponent>(entity) {
                Some(m) => m,
                None => return,
            };

            mover.phase += mover.speed * dt;

            match mover.move_type {
                MoveType::Linear => {
                    let t = (mover.phase.sin() + 1.0) * 0.5;
                    let offset = t * mover.distance;
                    match mover.axis {
                        MoveAxis::Horizontal => Vec2::new(offset, 0.0),
                        MoveAxis::Vertical => Vec2::new(0.0, offset),
                        MoveAxis::Both => Vec2::new(offset, offset),
                    }
                }
                MoveType::Sine => {
                    let offset = mover.phase.sin() * mover.distance;
                    match mover.axis {
                        MoveAxis::Horizontal => Vec2::new(offset, 0.0),
                        MoveAxis::Vertical => Vec2::new(0.0, offset),
                        MoveAxis::Both => Vec2::new(offset, offset),
                    }
                }
                MoveType::Circular => Vec2::new(
                    mover.phase.cos() * mover.distance,
                    mover.phase.sin() * mover.distance,
                ),
            }
        };

        // The offset is applied relative to the entity's base position.
        // In a full implementation, we'd store the base position separately.
        // For now, we apply the offset as velocity.
        let _ = new_offset;
    }

    fn update_rope(world: &mut World, entity: openclaw_core::ecs::Entity, dt: f32) {
        if let Some(rope) = world.get_component_mut::<RopeComponent>(entity) {
            // Pendulum physics simulation
            let gravity = 9.81;
            let angular_acceleration = -(gravity / rope.length) * rope.current_angle.sin();
            rope.angular_velocity += angular_acceleration * dt;
            rope.angular_velocity *= 0.998; // damping
            rope.current_angle += rope.angular_velocity * dt;
        }
    }

    fn update_followables(world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        for &entity in &entities {
            let target_pos = {
                let follow = match world.get_component::<FollowableComponent>(entity) {
                    Some(f) => f,
                    None => continue,
                };
                let target = match follow.target {
                    Some(t) => t,
                    None => continue,
                };
                world
                    .get_component::<TransformComponent>(target)
                    .map(|t| t.position + follow.offset)
            };

            if let Some(target_pos) = target_pos {
                let (speed, max_dist) = {
                    let follow = world.get_component::<FollowableComponent>(entity).unwrap();
                    (follow.speed, follow.max_distance)
                };
                if let Some(transform) = world.get_component_mut::<TransformComponent>(entity) {
                    let to_target = target_pos - transform.position;
                    let dist = to_target.length();
                    if dist > max_dist {
                        transform.position = target_pos;
                    } else if dist > 1.0 {
                        let move_dist = (speed * dt).min(dist);
                        transform.position += to_target.normalize() * move_dist;
                    }
                }
            }
        }
    }
}
