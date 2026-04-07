//! Audio engine trait and null implementation.

use super::music::MusicId;
use super::sound::{SoundHandle, SoundId};

/// Trait for audio playback backends.
pub trait AudioEngine: Send + Sync {
    /// Play a sound effect and return a handle to control it.
    fn play_sound(&mut self, sound_id: SoundId, volume: f32, looping: bool) -> SoundHandle;

    /// Start playing background music. Only one music track plays at a time.
    fn play_music(&mut self, music_id: MusicId, volume: f32, looping: bool);

    /// Stop the currently playing music.
    fn stop_music(&mut self);

    /// Stop a specific sound effect by its handle.
    fn stop_sound(&mut self, handle: SoundHandle);

    /// Set the master volume (0.0 = silent, 1.0 = full).
    fn set_master_volume(&mut self, volume: f32);

    /// Set the music volume (0.0 = silent, 1.0 = full).
    fn set_music_volume(&mut self, volume: f32);

    /// Set the sound effects volume (0.0 = silent, 1.0 = full).
    fn set_sfx_volume(&mut self, volume: f32);

    /// Get the current master volume.
    fn master_volume(&self) -> f32;

    /// Get the current music volume.
    fn music_volume(&self) -> f32;

    /// Get the current SFX volume.
    fn sfx_volume(&self) -> f32;
}

/// A no-op audio engine that silently ignores all calls. Useful for headless testing
/// or when no audio device is available.
pub struct NullAudioEngine {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    next_handle: u32,
}

impl NullAudioEngine {
    /// Create a new null audio engine with default volume levels.
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            next_handle: 0,
        }
    }
}

impl Default for NullAudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine for NullAudioEngine {
    fn play_sound(&mut self, _sound_id: SoundId, _volume: f32, _looping: bool) -> SoundHandle {
        let handle = SoundHandle(self.next_handle);
        self.next_handle += 1;
        tracing::trace!("NullAudioEngine::play_sound -> {:?}", handle);
        handle
    }

    fn play_music(&mut self, music_id: MusicId, _volume: f32, _looping: bool) {
        tracing::trace!("NullAudioEngine::play_music({:?})", music_id);
    }

    fn stop_music(&mut self) {
        tracing::trace!("NullAudioEngine::stop_music()");
    }

    fn stop_sound(&mut self, handle: SoundHandle) {
        tracing::trace!("NullAudioEngine::stop_sound({:?})", handle);
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    fn master_volume(&self) -> f32 {
        self.master_volume
    }

    fn music_volume(&self) -> f32 {
        self.music_volume
    }

    fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }
}
