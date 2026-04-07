//! Physics system: steps the physics world and syncs entity positions.

use glam::Vec2;
use openbee_core::ecs::{System, World};

use crate::components::kinematic::KinematicComponent;
use crate::components::physics::{PhysicsBodyType, PhysicsComponent};
use crate::components::transform::TransformComponent;

/// Gravity constant in pixels per second squared.
const GRAVITY: f32 = 1200.0;

/// Updates the physics simulation and synchronizes entity transforms.
pub struct PhysicsSystem {
    /// Accumulated gravity applied to dynamic entities without a physics body.
    gravity: Vec2,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            gravity: Vec2::new(0.0, GRAVITY),
        }
    }
}

impl System for PhysicsSystem {
    fn name(&self) -> &str {
        "PhysicsSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        for entity in entities {
            // Apply gravity and kinematic movement for entities without physics bodies
            let has_physics_body = world
                .get_component::<PhysicsComponent>(entity)
                .map_or(false, |p| p.body_handle.is_some());

            if !has_physics_body {
                // Manual kinematic integration
                let gravity_scale = world
                    .get_component::<PhysicsComponent>(entity)
                    .map_or(0.0, |p| {
                        if p.body_type == PhysicsBodyType::Static {
                            0.0
                        } else {
                            p.gravity_scale
                        }
                    });

                if let Some(kinematic) = world.get_component_mut::<KinematicComponent>(entity) {
                    // Apply gravity
                    kinematic.velocity += self.gravity * gravity_scale * dt;

                    // Apply acceleration
                    kinematic.velocity += kinematic.acceleration * dt;

                    // Clamp to max speed
                    kinematic.velocity.x = kinematic.velocity.x
                        .clamp(-kinematic.max_speed.x, kinematic.max_speed.x);
                    kinematic.velocity.y = kinematic.velocity.y
                        .clamp(-kinematic.max_speed.y, kinematic.max_speed.y);

                    let velocity = kinematic.velocity;

                    // Update position
                    if let Some(transform) = world.get_component_mut::<TransformComponent>(entity) {
                        transform.position += velocity * dt;
                    }
                }
            }
        }
    }
}
