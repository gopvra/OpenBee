//! Spawner components for creating actors and projectiles.

pub mod actor_spawner;
pub mod projectile_spawner;

pub use actor_spawner::ActorSpawnerComponent;
pub use projectile_spawner::{ProjectileSpawnerComponent, ProjectileType};
