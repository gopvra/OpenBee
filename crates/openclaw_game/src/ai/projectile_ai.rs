//! Projectile AI behavior: straight, arcing, and homing trajectories.

use glam::Vec2;
use openclaw_core::ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

/// Trajectory type for a projectile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectileTrajectory {
    /// Travels in a straight line.
    Straight,
    /// Follows a parabolic arc (gravity-affected).
    Arcing,
    /// Tracks a target entity.
    Homing,
    /// Sinusoidal wave pattern.
    Wave,
}

/// AI component for projectile entities.
#[derive(Debug, Clone)]
pub struct ProjectileAiComponent {
    /// How the projectile moves.
    pub trajectory: ProjectileTrajectory,
    /// Movement direction (normalized).
    pub direction: Vec2,
    /// Speed in pixels per second.
    pub speed: f32,
    /// Damage dealt on impact.
    pub damage: i32,
    /// Entity that fired the projectile (to avoid self-damage).
    pub owner: Option<Entity>,
    /// Maximum lifetime in seconds before auto-despawn.
    pub lifetime: f32,
    /// Time alive so far.
    pub age: f32,
    /// Homing turn rate in radians per second (for Homing trajectory).
    pub homing_strength: f32,
    /// Target entity for homing projectiles.
    pub homing_target: Option<Entity>,
}

impl ProjectileAiComponent {
    /// Create a simple straight-line projectile.
    pub fn straight(direction: Vec2, speed: f32, damage: i32, owner: Option<Entity>) -> Self {
        Self {
            trajectory: ProjectileTrajectory::Straight,
            direction: direction.normalize_or_zero(),
            speed,
            damage,
            owner,
            lifetime: 5.0,
            age: 0.0,
            homing_strength: 0.0,
            homing_target: None,
        }
    }

    /// Create an arcing projectile (for dynamite).
    pub fn arcing(direction: Vec2, speed: f32, damage: i32, owner: Option<Entity>) -> Self {
        Self {
            trajectory: ProjectileTrajectory::Arcing,
            direction: direction.normalize_or_zero(),
            speed,
            damage,
            owner,
            lifetime: 8.0,
            age: 0.0,
            homing_strength: 0.0,
            homing_target: None,
        }
    }

    /// Create a homing projectile (for magic).
    pub fn homing(
        direction: Vec2,
        speed: f32,
        damage: i32,
        owner: Option<Entity>,
        target: Option<Entity>,
    ) -> Self {
        Self {
            trajectory: ProjectileTrajectory::Homing,
            direction: direction.normalize_or_zero(),
            speed,
            damage,
            owner,
            lifetime: 6.0,
            age: 0.0,
            homing_strength: 3.0,
            homing_target: target,
        }
    }

    /// Whether the projectile has exceeded its lifetime.
    pub fn is_expired(&self) -> bool {
        self.age >= self.lifetime
    }

    /// Tick the lifetime counter.
    pub fn tick(&mut self, dt: f32) {
        self.age += dt;
    }
}

impl Component for ProjectileAiComponent {}
