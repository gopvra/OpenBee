//! Music and MIDI data types.

/// Unique identifier for a loaded music track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MusicId(pub u32);

/// Raw audio data for music playback (e.g., decoded OGG/WAV).
#[derive(Debug, Clone)]
pub struct MusicData {
    /// Raw PCM sample data.
    pub samples: Vec<i16>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels.
    pub channels: u16,
}

impl MusicData {
    /// Create new music data.
    pub fn new(samples: Vec<i16>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
        }
    }

    /// Duration of the music in seconds.
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 || self.channels == 0 {
            return 0.0;
        }
        self.samples.len() as f32 / (self.sample_rate as f32 * self.channels as f32)
    }
}

/// MIDI data for music playback using the original Captain Claw MIDI tracks.
#[derive(Debug, Clone)]
pub struct MidiData {
    /// Raw MIDI file bytes.
    pub raw_bytes: Vec<u8>,
    /// Track name if available.
    pub name: Option<String>,
}

impl MidiData {
    /// Create new MIDI data from raw bytes.
    pub fn new(raw_bytes: Vec<u8>) -> Self {
        Self {
            raw_bytes,
            name: None,
        }
    }

    /// Create new MIDI data with a name.
    pub fn with_name(raw_bytes: Vec<u8>, name: String) -> Self {
        Self {
            raw_bytes,
            name: Some(name),
        }
    }

    /// Return the size of the raw data in bytes.
    pub fn size_bytes(&self) -> usize {
        self.raw_bytes.len()
    }
}
