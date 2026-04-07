//! Sound component for entities that emit sound effects.

use openclaw_core::ecs::Component;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Handle to a currently playing sound instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SoundHandle(pub u64);

/// Maps named events to sound asset paths and tracks playback.
#[derive(Debug, Clone)]
pub struct SoundComponent {
    /// Map from event name (e.g. "jump", "attack") to sound asset path.
    pub sounds: FxHashMap<String, String>,
    /// Handle of the currently playing sound, if any.
    pub current_sound: Option<SoundHandle>,
}

impl Default for SoundComponent {
    fn default() -> Self {
        Self {
            sounds: FxHashMap::default(),
            current_sound: None,
        }
    }
}

impl Component for SoundComponent {}
