//! Glitter/particle effect component.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Type of glitter/sparkle particle effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlitterType {
    /// Small sparkle effect (treasures).
    Sparkle,
    /// Dust particles (ground impact).
    Dust,
    /// Blood particles (combat).
    Blood,
    /// Fire particles.
    Fire,
    /// Magic/energy particles.
    Magic,
    /// Water splash particles.
    Water,
}

/// Emits decorative particle effects from an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlitterComponent {
    /// Type of particle to emit.
    pub particle_type: GlitterType,
    /// Emission rate in particles per second.
    pub rate: f32,
    /// Spread angle in radians.
    pub spread: f32,
    /// RGBA color tint for particles.
    pub color: [u8; 4],
}

impl Default for GlitterComponent {
    fn default() -> Self {
        Self {
            particle_type: GlitterType::Sparkle,
            rate: 10.0,
            spread: 0.5,
            color: [255, 255, 255, 255],
        }
    }
}

impl Component for GlitterComponent {}
