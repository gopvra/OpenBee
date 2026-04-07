//! Audio system for sound effects, music, and spatial audio.

pub mod audio_engine;
pub mod music;
pub mod sound;
pub mod spatial;

pub use audio_engine::{AudioEngine, NullAudioEngine};
pub use music::{MidiData, MusicData, MusicId};
pub use sound::{SoundData, SoundHandle, SoundId};
pub use spatial::{SpatialAudioSource, SpatialAudioSystem};
