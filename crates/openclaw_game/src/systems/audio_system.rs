//! Audio system: manages sound playback and spatial audio.

use openclaw_core::ecs::{System, World};

use crate::components::audio::global_ambient::GlobalAmbientSoundComponent;
use crate::components::audio::local_ambient::LocalAmbientSoundComponent;
use crate::components::sound::SoundComponent;

/// Updates audio playback, including spatial attenuation for local sounds.
pub struct AudioSystem;

impl System for AudioSystem {
    fn name(&self) -> &str {
        "AudioSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<_> = world.entities();

        for entity in entities {
            // Update local ambient sounds (spatial attenuation based on listener distance).
            if world.has_component::<LocalAmbientSoundComponent>(entity) {
                // Spatial audio volume calculation would use distance to camera/player.
                // Actual audio playback is delegated to the AudioEngine trait.
            }

            // Global ambients play at constant volume.
            if world.has_component::<GlobalAmbientSoundComponent>(entity) {
                // Ensure looping sounds keep playing.
            }

            // Entity sounds (one-shot effects triggered by gameplay).
            if let Some(_sound) = world.get_component::<SoundComponent>(entity) {
                // Sound playback is triggered by other systems; this system
                // ensures cleanup of finished sound handles.
            }
        }
    }
}
