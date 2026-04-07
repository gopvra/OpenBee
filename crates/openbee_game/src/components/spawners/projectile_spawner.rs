//! Projectile spawner component for ranged attacks.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Types of projectiles that can be spawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectileType {
    /// Standard pistol bullet.
    PistolBullet,
    /// Dynamite stick (arcing trajectory).
    Dynamite,
    /// Magic projectile (homing).
    MagicBolt,
    /// Enemy arrow.
    Arrow,
    /// Enemy spear throw.
    Spear,
    /// Enemy fireball.
    Fireball,
    /// Cannonball from ship cannons.
    Cannonball,
    /// Rock thrown by enemies.
    Rock,
}

/// An entity that spawns projectiles at a configured rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileSpawnerComponent {
    /// Type of projectile to fire.
    pub projectile_type: ProjectileType,
    /// Minimum time between shots in seconds.
    pub fire_rate: f32,
    /// Speed of the projectile in pixels per second.
    pub speed: f32,
    /// Damage dealt by each projectile.
    pub damage: i32,
    /// Current timer until next allowed shot.
    pub timer: f32,
}

impl Default for ProjectileSpawnerComponent {
    fn default() -> Self {
        Self {
            projectile_type: ProjectileType::PistolBullet,
            fire_rate: 0.5,
            speed: 400.0,
            damage: 5,
            timer: 0.0,
        }
    }
}

impl Component for ProjectileSpawnerComponent {}
