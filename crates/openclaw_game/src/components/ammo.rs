//! Ammo component tracking ranged weapon ammunition.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Ammunition counts for the three ranged weapons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmoComponent {
    pub pistol: i32,
    pub dynamite: i32,
    pub magic: i32,
    pub max_pistol: i32,
    pub max_dynamite: i32,
    pub max_magic: i32,
}

impl AmmoComponent {
    /// Add pistol ammo, clamped to maximum.
    pub fn add_pistol(&mut self, amount: i32) {
        self.pistol = (self.pistol + amount).min(self.max_pistol);
    }

    /// Add dynamite ammo, clamped to maximum.
    pub fn add_dynamite(&mut self, amount: i32) {
        self.dynamite = (self.dynamite + amount).min(self.max_dynamite);
    }

    /// Add magic ammo, clamped to maximum.
    pub fn add_magic(&mut self, amount: i32) {
        self.magic = (self.magic + amount).min(self.max_magic);
    }
}

impl Default for AmmoComponent {
    fn default() -> Self {
        Self {
            pistol: 10,
            dynamite: 3,
            magic: 5,
            max_pistol: 99,
            max_dynamite: 25,
            max_magic: 25,
        }
    }
}

impl Component for AmmoComponent {}
