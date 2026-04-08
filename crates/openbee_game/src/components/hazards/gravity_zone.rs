//! Gravity zone component for areas with modified gravitational behaviour.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A spatial zone that modifies the gravitational force applied to entities within it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityZoneComponent {
    /// How gravity is modified within the zone.
    pub zone_type: GravityZoneType,
    /// Strength multiplier for the gravitational force.
    pub strength: f32,
    /// Normalized direction vector for the gravity (x, y).
    pub direction: (f32, f32),
    /// Bounding rectangle of the zone: (x, y, width, height).
    pub zone_rect: (f32, f32, f32, f32),
    /// Whether this zone affects player entities.
    pub affect_players: bool,
    /// Whether this zone affects enemy entities.
    pub affect_enemies: bool,
    /// Whether this zone affects projectile entities.
    pub affect_projectiles: bool,
    /// How quickly gravity transitions when an entity enters the zone (0.0 = instant, 1.0 = very smooth).
    pub transition_smoothness: f32,
}

/// The type of gravity modification applied by a gravity zone.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GravityZoneType {
    /// Completely replaces the world gravity with the zone's gravity.
    Override,
    /// Adds the zone's gravity on top of the existing world gravity.
    Additive,
    /// Reverses the existing world gravity.
    Reverse,
    /// Sets gravity to zero within the zone.
    Zero,
    /// Gravity points toward a specific point.
    Radial { center_x: f32, center_y: f32 },
}

impl Default for GravityZoneComponent {
    fn default() -> Self {
        Self {
            zone_type: GravityZoneType::Override,
            strength: 1.0,
            direction: (0.0, 1.0),
            zone_rect: (0.0, 0.0, 256.0, 256.0),
            affect_players: true,
            affect_enemies: true,
            affect_projectiles: false,
            transition_smoothness: 0.0,
        }
    }
}

impl Component for GravityZoneComponent {}
