//! Local ambient sound component for positional audio.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// An ambient sound that plays within a radius of the entity's position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAmbientSoundComponent {
    /// Path to the sound asset.
    pub sound_path: String,
    /// Maximum audible radius in pixels.
    pub radius: f32,
    /// Base volume at the entity's position (0.0 to 1.0).
    pub volume: f32,
    /// Whether the sound loops.
    pub looping: bool,
}

impl Default for LocalAmbientSoundComponent {
    fn default() -> Self {
        Self {
            sound_path: String::new(),
            radius: 300.0,
            volume: 1.0,
            looping: true,
        }
    }
}

impl Component for LocalAmbientSoundComponent {}
