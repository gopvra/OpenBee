//! Sound effect types.

/// Unique identifier for a loaded sound effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub u32);

/// Handle to a playing sound instance, used to stop or modify playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u32);

/// Raw audio sample data for a sound effect.
#[derive(Debug, Clone)]
pub struct SoundData {
    /// Raw PCM sample data (interleaved if multi-channel).
    pub samples: Vec<i16>,
    /// Sample rate in Hz (e.g., 44100).
    pub sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
}

impl SoundData {
    /// Create new sound data.
    pub fn new(samples: Vec<i16>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
        }
    }

    /// Duration of the sound in seconds.
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 || self.channels == 0 {
            return 0.0;
        }
        self.samples.len() as f32 / (self.sample_rate as f32 * self.channels as f32)
    }

    /// Total size of the sample data in bytes.
    pub fn size_bytes(&self) -> usize {
        self.samples.len() * std::mem::size_of::<i16>()
    }
}
