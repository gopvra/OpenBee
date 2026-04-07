//! Game-specific events for the Captain Claw engine.

use crate::ecs::Entity;
use crate::events::event_bus::Event;
use glam::Vec2;

/// An actor (enemy, NPC, projectile, etc.) was created.
#[derive(Debug, Clone)]
pub struct ActorCreated {
    pub entity: Entity,
    pub actor_type: String,
    pub position: Vec2,
}
impl Event for ActorCreated {}

/// An actor was destroyed/removed from the world.
#[derive(Debug, Clone)]
pub struct ActorDestroyed {
    pub entity: Entity,
    pub reason: String,
}
impl Event for ActorDestroyed {}

/// An actor moved to a new position.
#[derive(Debug, Clone)]
pub struct ActorMoved {
    pub entity: Entity,
    pub old_position: Vec2,
    pub new_position: Vec2,
}
impl Event for ActorMoved {}

/// Damage was dealt to an entity.
#[derive(Debug, Clone)]
pub struct DamageDealt {
    pub attacker: Option<Entity>,
    pub target: Entity,
    pub amount: i32,
    pub damage_type: String,
}
impl Event for DamageDealt {}

/// An item was picked up by the player.
#[derive(Debug, Clone)]
pub struct ItemPickedUp {
    pub player: Entity,
    pub item: Entity,
    pub item_type: String,
    pub value: i32,
}
impl Event for ItemPickedUp {}

/// The player reached a checkpoint.
#[derive(Debug, Clone)]
pub struct CheckpointReached {
    pub player: Entity,
    pub checkpoint_id: u32,
    pub position: Vec2,
}
impl Event for CheckpointReached {}

/// The current level was completed.
#[derive(Debug, Clone)]
pub struct LevelCompleted {
    pub level_id: u32,
    pub score: u32,
    pub time_secs: f32,
}
impl Event for LevelCompleted {}

/// A new level started loading/playing.
#[derive(Debug, Clone)]
pub struct LevelStarted {
    pub level_id: u32,
    pub level_name: String,
}
impl Event for LevelStarted {}

/// A boss enemy was defeated.
#[derive(Debug, Clone)]
pub struct BossDefeated {
    pub boss_entity: Entity,
    pub boss_name: String,
    pub level_id: u32,
}
impl Event for BossDefeated {}

/// The player died.
#[derive(Debug, Clone)]
pub struct PlayerDied {
    pub player: Entity,
    pub cause: String,
    pub position: Vec2,
}
impl Event for PlayerDied {}

/// The player respawned.
#[derive(Debug, Clone)]
pub struct PlayerRespawned {
    pub player: Entity,
    pub checkpoint_id: u32,
    pub position: Vec2,
}
impl Event for PlayerRespawned {}

/// The player's score changed.
#[derive(Debug, Clone)]
pub struct ScoreChanged {
    pub old_score: u32,
    pub new_score: u32,
    pub reason: String,
}
impl Event for ScoreChanged {}

/// The player's remaining lives changed.
#[derive(Debug, Clone)]
pub struct LivesChanged {
    pub old_lives: i32,
    pub new_lives: i32,
}
impl Event for LivesChanged {}

/// The player's ammo count changed.
#[derive(Debug, Clone)]
pub struct AmmoChanged {
    pub weapon_type: String,
    pub old_ammo: i32,
    pub new_ammo: i32,
}
impl Event for AmmoChanged {}

/// A powerup effect was activated on the player.
#[derive(Debug, Clone)]
pub struct PowerupActivated {
    pub player: Entity,
    pub powerup_type: String,
    pub duration_secs: f32,
}
impl Event for PowerupActivated {}

/// A powerup effect expired.
#[derive(Debug, Clone)]
pub struct PowerupExpired {
    pub player: Entity,
    pub powerup_type: String,
}
impl Event for PowerupExpired {}

/// The overall game state changed (e.g., menu -> playing -> paused).
#[derive(Debug, Clone)]
pub struct GameStateChanged {
    pub old_state: String,
    pub new_state: String,
}
impl Event for GameStateChanged {}

/// A save-game was requested.
#[derive(Debug, Clone)]
pub struct SaveGameRequested {
    pub slot: u32,
}
impl Event for SaveGameRequested {}

/// A load-game was requested.
#[derive(Debug, Clone)]
pub struct LoadGameRequested {
    pub slot: u32,
}
impl Event for LoadGameRequested {}
