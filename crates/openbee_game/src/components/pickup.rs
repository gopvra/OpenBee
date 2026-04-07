//! Pickup component for collectable items.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

use crate::components::powerup::PowerupType;

/// Types of treasure with different point values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TreasureType {
    Coin,
    GoldBar,
    Ring,
    Chalice,
    Crown,
    Scepter,
    Cross,
    Gecko,
}

impl TreasureType {
    /// Base score value for this treasure type.
    pub fn base_value(self) -> u64 {
        match self {
            Self::Coin => 100,
            Self::GoldBar => 500,
            Self::Ring => 1500,
            Self::Chalice => 2500,
            Self::Crown => 5000,
            Self::Scepter => 7500,
            Self::Cross => 10000,
            Self::Gecko => 25000,
        }
    }
}

/// Types of ammunition pickups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmmoType {
    Pistol,
    Dynamite,
    Magic,
}

/// What kind of pickup this entity represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PickupType {
    /// Treasure worth score points.
    Treasure(TreasureType),
    /// Health restoration.
    Health,
    /// Ammunition for a weapon type.
    Ammo(AmmoType),
    /// A timed powerup.
    Powerup(PowerupType),
    /// An extra life.
    ExtraLife,
    /// A piece of the treasure map.
    MapPiece,
    /// A gem required to complete the level.
    Gem,
}

/// An item that the player can pick up by touching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickupComponent {
    /// What type of pickup this is.
    pub pickup_type: PickupType,
    /// Numeric value (health amount, ammo count, score, etc.).
    pub value: i32,
    /// Sound to play when picked up.
    pub sound: Option<String>,
}

impl Default for PickupComponent {
    fn default() -> Self {
        Self {
            pickup_type: PickupType::Treasure(TreasureType::Coin),
            value: 100,
            sound: None,
        }
    }
}

impl Component for PickupComponent {}
