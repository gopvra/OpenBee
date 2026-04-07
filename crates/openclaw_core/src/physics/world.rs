//! Physics world wrapping Rapier2D simulation.

use glam::Vec2;
use rapier2d::prelude::*;

use super::body::{BodyDesc, BodyHandle, BodyShape};
use super::PIXELS_PER_METER;

/// Wrapper around the Rapier2D physics pipeline, providing a simplified API
/// in pixel-space coordinates.
pub struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl PhysicsWorld {
    /// Create a new physics world with the given gravity in pixels/second^2.
    pub fn new(gravity_pixels: Vec2) -> Self {
        let gravity = vector![
            gravity_pixels.x / PIXELS_PER_METER,
            gravity_pixels.y / PIXELS_PER_METER
        ];
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    /// Step the physics simulation by the given delta time in seconds.
    pub fn step(&mut self, dt: f32) {
        self.integration_parameters.dt = dt;
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
    }

    /// Create a dynamic rigid body with the given description and return its handle.
    pub fn create_dynamic_body(&mut self, desc: &BodyDesc) -> BodyHandle {
        let pos = desc.position / PIXELS_PER_METER;
        let body = RigidBodyBuilder::dynamic()
            .translation(vector![pos.x, pos.y])
            .linear_damping(desc.linear_damping)
            .gravity_scale(desc.gravity_scale)
            .build();
        let handle = self.rigid_body_set.insert(body);
        self.attach_collider(handle, desc);
        BodyHandle(handle)
    }

    /// Create a static (immovable) rigid body.
    pub fn create_static_body(&mut self, desc: &BodyDesc) -> BodyHandle {
        let pos = desc.position / PIXELS_PER_METER;
        let body = RigidBodyBuilder::fixed()
            .translation(vector![pos.x, pos.y])
            .build();
        let handle = self.rigid_body_set.insert(body);
        self.attach_collider(handle, desc);
        BodyHandle(handle)
    }

    /// Create a kinematic (position-based) rigid body.
    pub fn create_kinematic_body(&mut self, desc: &BodyDesc) -> BodyHandle {
        let pos = desc.position / PIXELS_PER_METER;
        let body = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![pos.x, pos.y])
            .build();
        let handle = self.rigid_body_set.insert(body);
        self.attach_collider(handle, desc);
        BodyHandle(handle)
    }

    /// Remove a body and all its attached colliders from the world.
    pub fn remove_body(&mut self, handle: BodyHandle) {
        self.rigid_body_set.remove(
            handle.0,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
    }

    /// Apply a force (in pixel-space newtons) to a body. The force is applied continuously
    /// and will be cleared after the next physics step.
    pub fn apply_force(&mut self, handle: BodyHandle, force_pixels: Vec2) {
        if let Some(body) = self.rigid_body_set.get_mut(handle.0) {
            let force = force_pixels / PIXELS_PER_METER;
            body.add_force(vector![force.x, force.y], true);
        }
    }

    /// Apply an instantaneous impulse (in pixel-space) to a body.
    pub fn apply_impulse(&mut self, handle: BodyHandle, impulse_pixels: Vec2) {
        if let Some(body) = self.rigid_body_set.get_mut(handle.0) {
            let impulse = impulse_pixels / PIXELS_PER_METER;
            body.apply_impulse(vector![impulse.x, impulse.y], true);
        }
    }

    /// Set the linear velocity of a body (in pixels/second).
    pub fn set_velocity(&mut self, handle: BodyHandle, velocity_pixels: Vec2) {
        if let Some(body) = self.rigid_body_set.get_mut(handle.0) {
            let vel = velocity_pixels / PIXELS_PER_METER;
            body.set_linvel(vector![vel.x, vel.y], true);
        }
    }

    /// Get the current position of a body in pixel coordinates.
    pub fn get_position(&self, handle: BodyHandle) -> Option<Vec2> {
        self.rigid_body_set.get(handle.0).map(|body| {
            let pos = body.translation();
            Vec2::new(pos.x * PIXELS_PER_METER, pos.y * PIXELS_PER_METER)
        })
    }

    /// Get the current linear velocity of a body in pixels/second.
    pub fn get_velocity(&self, handle: BodyHandle) -> Option<Vec2> {
        self.rigid_body_set.get(handle.0).map(|body| {
            let vel = body.linvel();
            Vec2::new(vel.x * PIXELS_PER_METER, vel.y * PIXELS_PER_METER)
        })
    }

    /// Set the gravity for the physics world (in pixels/second^2).
    pub fn set_gravity(&mut self, gravity_pixels: Vec2) {
        self.gravity = vector![
            gravity_pixels.x / PIXELS_PER_METER,
            gravity_pixels.y / PIXELS_PER_METER
        ];
    }

    /// Access the underlying rigid body set.
    pub fn rigid_body_set(&self) -> &RigidBodySet {
        &self.rigid_body_set
    }

    /// Access the underlying collider set.
    pub fn collider_set(&self) -> &ColliderSet {
        &self.collider_set
    }

    /// Create and attach a collider to a body based on the body description.
    fn attach_collider(&mut self, body_handle: RigidBodyHandle, desc: &BodyDesc) {
        let shape: SharedShape = match &desc.shape {
            BodyShape::Circle { radius } => SharedShape::ball(*radius / PIXELS_PER_METER),
            BodyShape::Rectangle {
                half_width,
                half_height,
            } => SharedShape::cuboid(
                *half_width / PIXELS_PER_METER,
                *half_height / PIXELS_PER_METER,
            ),
            BodyShape::Capsule {
                half_height,
                radius,
            } => SharedShape::capsule_y(
                *half_height / PIXELS_PER_METER,
                *radius / PIXELS_PER_METER,
            ),
        };

        let collider = ColliderBuilder::new(shape)
            .friction(desc.friction)
            .restitution(desc.restitution)
            .sensor(desc.is_sensor)
            .build();

        self.collider_set
            .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
    }
}
