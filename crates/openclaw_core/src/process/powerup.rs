//! Powerup process: a timed effect that publishes an expiry event when done.

use crate::ecs::Entity;

use super::process::{Process, ProcessState};

/// A timed powerup effect. When the duration expires, the process succeeds
/// and signals that the powerup has ended.
pub struct PowerupProcess {
    /// The entity that has the powerup.
    pub player: Entity,
    /// Type of powerup (e.g., "invincibility", "fire_sword", "catnip").
    pub powerup_type: String,
    /// Total duration of the effect in seconds.
    pub duration: f32,
    /// Time remaining in seconds.
    pub remaining: f32,
    /// Whether the expiry event has been flagged (checked externally).
    pub expired: bool,
}

impl PowerupProcess {
    /// Create a new powerup process.
    pub fn new(player: Entity, powerup_type: impl Into<String>, duration: f32) -> Self {
        Self {
            player,
            powerup_type: powerup_type.into(),
            duration,
            remaining: duration,
            expired: false,
        }
    }

    /// Return the fraction of time remaining (1.0 = just started, 0.0 = about to expire).
    pub fn remaining_ratio(&self) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else {
            (self.remaining / self.duration).clamp(0.0, 1.0)
        }
    }
}

impl Process for PowerupProcess {
    fn on_init(&mut self) {
        tracing::info!(
            "PowerupProcess: {} activated on {:?} for {:.1}s",
            self.powerup_type,
            self.player,
            self.duration
        );
    }

    fn update(&mut self, dt: f32) -> ProcessState {
        self.remaining -= dt;
        if self.remaining <= 0.0 {
            self.expired = true;
            ProcessState::Succeeded
        } else {
            ProcessState::Running
        }
    }

    fn on_success(&mut self) {
        tracing::info!(
            "PowerupProcess: {} expired on {:?}",
            self.powerup_type,
            self.player
        );
        // In a full implementation, we would publish a PowerupExpired event here
        // via the event bus. The caller should check `expired` and do so.
    }

    fn on_abort(&mut self) {
        tracing::info!(
            "PowerupProcess: {} aborted on {:?}",
            self.powerup_type,
            self.player
        );
    }

    fn name(&self) -> &str {
        "PowerupProcess"
    }
}
