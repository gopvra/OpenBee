//! Sound trigger component for playing sounds when entering an area.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A trigger that plays a sound effect when the player enters its area.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundTriggerComponent {
    /// Path to the sound asset.
    pub sound_path: String,
    /// Playback volume (0.0 to 1.0).
    pub volume: f32,
    /// Whether the sound has already been played (for one-shot triggers).
    pub played: bool,
}

impl Default for SoundTriggerComponent {
    fn default() -> Self {
        Self {
            sound_path: String::new(),
            volume: 1.0,
            played: false,
        }
    }
}

impl Component for SoundTriggerComponent {}
