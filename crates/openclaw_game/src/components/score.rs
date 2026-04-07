//! Score component for tracking points.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Tracks the score and any active multiplier for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreComponent {
    /// Total accumulated score.
    pub score: u64,
    /// Score multiplier (1.0 = normal).
    pub multiplier: f32,
}

impl ScoreComponent {
    /// Add points with the current multiplier applied.
    pub fn add(&mut self, base_points: u64) {
        self.score += (base_points as f32 * self.multiplier) as u64;
    }
}

impl Default for ScoreComponent {
    fn default() -> Self {
        Self {
            score: 0,
            multiplier: 1.0,
        }
    }
}

impl Component for ScoreComponent {}
