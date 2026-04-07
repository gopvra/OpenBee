//! Health component for damageable entities.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Tracks hit points and invulnerability state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthComponent {
    /// Current health points.
    pub current: i32,
    /// Maximum health points.
    pub max: i32,
    /// Whether the entity is currently invulnerable.
    pub invulnerable: bool,
    /// Time remaining for invulnerability in seconds (after being hit).
    pub invulnerability_timer: f32,
}

impl HealthComponent {
    /// Apply damage, respecting invulnerability. Returns actual damage dealt.
    pub fn apply_damage(&mut self, amount: i32) -> i32 {
        if self.invulnerable || amount <= 0 {
            return 0;
        }
        let actual = amount.min(self.current);
        self.current -= actual;
        actual
    }

    /// Heal by the given amount, clamped to max.
    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Whether the entity is dead (zero or negative health).
    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }

    /// Health as a fraction of max (0.0 to 1.0).
    pub fn fraction(&self) -> f32 {
        if self.max <= 0 {
            return 0.0;
        }
        self.current as f32 / self.max as f32
    }
}

impl Default for HealthComponent {
    fn default() -> Self {
        Self {
            current: 100,
            max: 100,
            invulnerable: false,
            invulnerability_timer: 0.0,
        }
    }
}

impl Component for HealthComponent {}
