//! Destructible terrain component for breakable environmental tiles.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Terrain that can be damaged and destroyed, optionally respawning after a delay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructibleTerrainComponent {
    /// Current hit points.
    pub health: i32,
    /// Maximum hit points.
    pub max_health: i32,
    /// Whether this terrain has been destroyed.
    pub destroyed: bool,
    /// Minimum damage required to affect this terrain (attacks below this are ignored).
    pub damage_threshold: i32,
    /// Visual style of the destruction effect.
    pub destruction_type: DestructionType,
    /// If `Some`, the terrain respawns after this many seconds.
    pub respawn_time: Option<f32>,
    /// Remaining time until respawn (counts down while destroyed).
    pub respawn_timer: f32,
    /// Number of debris particles to spawn on destruction.
    pub debris_count: u32,
    /// Optional sprite name for debris particles.
    pub debris_sprite: Option<String>,
    /// Optional sound effect to play on destruction.
    pub destroy_sound: Option<String>,
    /// Template names of items/pickups to spawn when destroyed.
    pub loot_on_destroy: Vec<String>,
    /// Tile index to switch to when destroyed (for `Replace` destruction type).
    pub tile_index_destroyed: Option<u32>,
}

/// Visual style of the terrain destruction effect.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DestructionType {
    /// Falls apart into debris particles.
    Crumble,
    /// Explosion effect with radial debris.
    Explode,
    /// Fades out gradually.
    Fade,
    /// Replaced with a different tile.
    Replace,
    /// Collapses downward.
    Collapse,
}

impl Default for DestructibleTerrainComponent {
    fn default() -> Self {
        Self {
            health: 100,
            max_health: 100,
            destroyed: false,
            damage_threshold: 0,
            destruction_type: DestructionType::Crumble,
            respawn_time: None,
            respawn_timer: 0.0,
            debris_count: 5,
            debris_sprite: None,
            destroy_sound: None,
            loot_on_destroy: Vec::new(),
            tile_index_destroyed: None,
        }
    }
}

impl DestructibleTerrainComponent {
    /// Apply `amount` damage to this terrain.
    ///
    /// Damage below `damage_threshold` is ignored. Returns `true` if this hit
    /// caused the terrain to be destroyed.
    pub fn take_damage(&mut self, amount: i32) -> bool {
        if self.destroyed {
            return false;
        }
        if amount < self.damage_threshold {
            return false;
        }
        self.health -= amount;
        if self.health <= 0 {
            self.health = 0;
            self.destroyed = true;
            if let Some(respawn) = self.respawn_time {
                self.respawn_timer = respawn;
            }
            true
        } else {
            false
        }
    }

    /// Fully repair this terrain, restoring it to its initial state.
    pub fn repair(&mut self) {
        self.health = self.max_health;
        self.destroyed = false;
        self.respawn_timer = 0.0;
    }

    /// Tick the respawn timer by `dt` seconds.
    ///
    /// Returns `true` if the terrain should respawn this frame.
    pub fn try_respawn(&mut self, dt: f32) -> bool {
        if !self.destroyed {
            return false;
        }
        if self.respawn_time.is_none() {
            return false;
        }
        self.respawn_timer -= dt;
        if self.respawn_timer <= 0.0 {
            self.repair();
            true
        } else {
            false
        }
    }
}

impl Component for DestructibleTerrainComponent {}
