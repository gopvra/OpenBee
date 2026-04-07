//! Inventory component for items, equipment, and collectibles.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Categories that items can belong to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ItemCategory {
    Weapon,
    Consumable,
    KeyItem,
    Treasure,
    Equipment,
    Ammo,
    Collectible,
}

/// Definition of an item type in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    /// Unique identifier for this item type.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description shown in inventory UI.
    pub description: String,
    /// Which category the item belongs to.
    pub category: ItemCategory,
    /// Maximum number that can stack in one slot.
    pub max_stack: u32,
    /// Optional icon resource path.
    pub icon: Option<String>,
    /// Effect when the item is used.
    pub use_effect: Option<ItemEffect>,
    /// Which equipment slot this item can go in, if any.
    pub equip_slot: Option<EquipSlot>,
    /// Gold value of the item.
    pub value: u32,
    /// Item rarity tier.
    pub rarity: ItemRarity,
}

/// Effect triggered when an item is used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemEffect {
    /// Restore health by the given amount.
    Heal(i32),
    /// Add ammunition of a specific type.
    AddAmmo { ammo_type: String, amount: i32 },
    /// Grant a temporary powerup.
    GrantPowerup { powerup: String, duration: f32 },
    /// Deal damage to the user (cursed items, etc.).
    Damage(i32),
    /// Teleport the user to a position.
    Teleport { x: f32, y: f32 },
    /// Custom effect identified by a string key.
    Custom(String),
}

/// Equipment slots that items can be placed in.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EquipSlot {
    Head,
    Body,
    Weapon,
    Accessory,
    Boots,
}

/// Rarity tiers for items.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// A single slot in the inventory holding a stack of one item type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    /// The item type id stored in this slot.
    pub item_id: String,
    /// How many of this item are in the slot.
    pub quantity: u32,
}

/// Inventory component attached to entities that can carry items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryComponent {
    /// Inventory slots; `None` means the slot is empty.
    pub slots: Vec<Option<InventorySlot>>,
    /// Maximum number of slots.
    pub max_slots: usize,
    /// Currently equipped items, keyed by slot name.
    pub equipped: HashMap<String, String>,
    /// Gold currency held.
    pub gold: u32,
    /// Whether items are automatically picked up on contact.
    pub auto_pickup: bool,
}

impl InventoryComponent {
    /// Create a new empty inventory with the given number of slots.
    pub fn new(max_slots: usize) -> Self {
        Self {
            slots: vec![None; max_slots],
            max_slots,
            equipped: HashMap::new(),
            gold: 0,
            auto_pickup: true,
        }
    }

    /// Add items to the inventory. Returns `false` if there is no room for all items.
    pub fn add_item(&mut self, item_id: &str, quantity: u32) -> bool {
        if quantity == 0 {
            return true;
        }

        let mut remaining = quantity;

        // First, try to stack onto existing slots with the same item.
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(ref mut inv_slot) = slot {
                if inv_slot.item_id == item_id {
                    let can_add = remaining; // no max_stack enforcement without item defs here
                    inv_slot.quantity += can_add;
                    remaining -= can_add;
                }
            }
        }

        // Then, fill empty slots.
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if slot.is_none() {
                *slot = Some(InventorySlot {
                    item_id: item_id.to_string(),
                    quantity: remaining,
                });
                remaining = 0;
            }
        }

        remaining == 0
    }

    /// Remove items from the inventory. Returns `false` if the entity doesn't have enough.
    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> bool {
        if quantity == 0 {
            return true;
        }

        let total = self.item_count(item_id);
        if total < quantity {
            return false;
        }

        let mut remaining = quantity;

        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(ref mut inv_slot) = slot {
                if inv_slot.item_id == item_id {
                    let take = remaining.min(inv_slot.quantity);
                    inv_slot.quantity -= take;
                    remaining -= take;
                    if inv_slot.quantity == 0 {
                        // Slot is now empty, will be cleared below.
                    }
                }
            }
        }

        // Clean up empty slots.
        for slot in self.slots.iter_mut() {
            if let Some(ref inv_slot) = slot {
                if inv_slot.quantity == 0 {
                    *slot = None;
                }
            }
        }

        true
    }

    /// Check whether the inventory contains at least one of the given item.
    pub fn has_item(&self, item_id: &str) -> bool {
        self.item_count(item_id) > 0
    }

    /// Return the total count of a given item across all slots.
    pub fn item_count(&self, item_id: &str) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item_id == item_id)
            .map(|s| s.quantity)
            .sum()
    }

    /// Equip an item into a named slot. Returns the previously equipped item id, if any.
    pub fn equip(&mut self, slot: &str, item_id: &str) -> Option<String> {
        let previous = self.equipped.insert(slot.to_string(), item_id.to_string());
        previous
    }

    /// Unequip the item in the given slot. Returns the item id that was there.
    pub fn unequip(&mut self, slot: &str) -> Option<String> {
        self.equipped.remove(slot)
    }

    /// Clear all items, equipment, and gold.
    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            *slot = None;
        }
        self.equipped.clear();
        self.gold = 0;
    }

    /// Whether every slot is occupied.
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }
}

impl Default for InventoryComponent {
    fn default() -> Self {
        Self::new(20)
    }
}

impl Component for InventoryComponent {}
