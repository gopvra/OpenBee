//! Loader for WAV audio files.

use super::ResourceLoader;
use crate::audio::sound::SoundData;
use anyhow::{bail, Result};
use std::any::Any;

/// Loader for `.wav` audio files.
#[derive(Clone)]
pub struct WavLoader;

impl ResourceLoader for WavLoader {
    fn extensions(&self) -> &[&str] {
        &["wav"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        // Minimal WAV header parsing.
        if data.len() < 44 {
            bail!("WAV file too small: {} ({} bytes)", path, data.len());
        }

        // Verify RIFF header.
        if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
            bail!("Not a valid WAV file: {}", path);
        }

        // Read format chunk.
        let channels = u16::from_le_bytes([data[22], data[23]]);
        let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let bits_per_sample = u16::from_le_bytes([data[34], data[35]]);

        // Find data chunk.
        let mut offset = 36;
        while offset + 8 < data.len() {
            let chunk_id = &data[offset..offset + 4];
            let chunk_size = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]) as usize;
            if chunk_id == b"data" {
                let audio_data =
                    &data[offset + 8..std::cmp::min(offset + 8 + chunk_size, data.len())];
                let samples = match bits_per_sample {
                    16 => audio_data
                        .chunks_exact(2)
                        .map(|c| i16::from_le_bytes([c[0], c[1]]))
                        .collect(),
                    8 => audio_data
                        .iter()
                        .map(|&b| ((b as i16) - 128) * 256)
                        .collect(),
                    _ => {
                        bail!(
                            "Unsupported bits per sample: {} in {}",
                            bits_per_sample,
                            path
                        );
                    }
                };

                tracing::debug!(
                    "WavLoader: loaded {} ({}Hz, {} ch, {} samples)",
                    path,
                    sample_rate,
                    channels,
                    audio_data.len() / (bits_per_sample as usize / 8)
                );

                return Ok(Box::new(SoundData::new(samples, sample_rate, channels)));
            }
            offset += 8 + chunk_size;
        }

        bail!("No data chunk found in WAV file: {}", path);
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
