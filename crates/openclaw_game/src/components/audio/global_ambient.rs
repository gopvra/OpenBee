//! Global ambient sound component for non-positional background audio.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A background ambient sound that plays at a constant volume regardless of position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAmbientSoundComponent {
    /// Path to the sound asset.
    pub sound_path: String,
    /// Playback volume (0.0 to 1.0).
    pub volume: f32,
    /// Whether the sound loops.
    pub looping: bool,
    /// Fade-in duration in seconds.
    pub fade_in: f32,
}

impl Default for GlobalAmbientSoundComponent {
    fn default() -> Self {
        Self {
            sound_path: String::new(),
            volume: 0.5,
            looping: true,
            fade_in: 1.0,
        }
    }
}

impl Component for GlobalAmbientSoundComponent {}
