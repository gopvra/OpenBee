//! Spatial audio: positional sound attenuation and panning.

use glam::Vec2;

/// A sound source with a position in the game world.
#[derive(Debug, Clone)]
pub struct SpatialAudioSource {
    /// Position in world pixels.
    pub position: Vec2,
    /// Maximum audible radius in pixels.
    pub radius: f32,
    /// Base volume before attenuation (0.0..=1.0).
    pub volume: f32,
}

impl SpatialAudioSource {
    /// Create a new spatial audio source.
    pub fn new(position: Vec2, radius: f32, volume: f32) -> Self {
        Self {
            position,
            radius,
            volume,
        }
    }
}

/// Result of spatial audio calculation for a single source.
#[derive(Debug, Clone, Copy)]
pub struct SpatialResult {
    /// Effective volume after distance attenuation (0.0..=1.0).
    pub volume: f32,
    /// Stereo pan value (-1.0 = full left, 0.0 = center, 1.0 = full right).
    pub pan: f32,
}

/// Manages the listener position and calculates volume/pan for spatial sources.
pub struct SpatialAudioSystem {
    /// Listener position in world pixels.
    pub listener_pos: Vec2,
    /// Width of the audible area around the listener (for pan calculation).
    pub listener_width: f32,
}

impl SpatialAudioSystem {
    /// Create a new spatial audio system with the listener at the origin.
    pub fn new() -> Self {
        Self {
            listener_pos: Vec2::ZERO,
            listener_width: 800.0, // default screen width
        }
    }

    /// Set the listener position.
    pub fn set_listener_position(&mut self, position: Vec2) {
        self.listener_pos = position;
    }

    /// Calculate the effective volume and stereo pan for a source relative to the listener.
    pub fn calculate_volume_and_pan(&self, source: &SpatialAudioSource) -> SpatialResult {
        let delta = source.position - self.listener_pos;
        let distance = delta.length();

        // Volume attenuation: linear falloff from source.radius.
        let volume = if distance >= source.radius {
            0.0
        } else if source.radius <= 0.0 {
            source.volume
        } else {
            source.volume * (1.0 - distance / source.radius)
        };

        // Pan: map horizontal offset to -1..1 range based on listener width.
        let half_width = self.listener_width * 0.5;
        let pan = if half_width <= 0.0 {
            0.0
        } else {
            (delta.x / half_width).clamp(-1.0, 1.0)
        };

        SpatialResult { volume, pan }
    }
}

impl Default for SpatialAudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_at_listener() {
        let system = SpatialAudioSystem::new();
        let source = SpatialAudioSource::new(Vec2::ZERO, 100.0, 1.0);
        let result = system.calculate_volume_and_pan(&source);
        assert!((result.volume - 1.0).abs() < 0.001);
        assert!((result.pan).abs() < 0.001);
    }

    #[test]
    fn test_spatial_out_of_range() {
        let system = SpatialAudioSystem::new();
        let source = SpatialAudioSource::new(Vec2::new(200.0, 0.0), 100.0, 1.0);
        let result = system.calculate_volume_and_pan(&source);
        assert!((result.volume).abs() < 0.001);
    }
}
