//! Loot component for entities that drop items on destruction.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A single item that can be dropped as loot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootItem {
    /// Template name of the item to spawn.
    pub template: String,
    /// Probability of dropping (0.0 to 1.0).
    pub chance: f32,
    /// Number of items to drop.
    pub count: u32,
}

/// Defines what an entity drops when destroyed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootComponent {
    /// List of possible loot drops.
    pub items: Vec<LootItem>,
    /// Global drop chance multiplier.
    pub drop_chance: f32,
}

impl Default for LootComponent {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            drop_chance: 1.0,
        }
    }
}

impl Component for LootComponent {}
