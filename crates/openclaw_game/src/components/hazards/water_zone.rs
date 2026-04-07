//! Water zone component for swimmable / submersible areas.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// A rectangular water zone that applies buoyancy, drag, and visual effects to
/// entities that enter it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterZoneComponent {
    /// Bounding rectangle of the zone: (x, y, width, height).
    pub zone_rect: (f32, f32, f32, f32),
    /// Y coordinate of the water surface (world-space).
    pub surface_y: f32,
    /// Direction of the water current.
    pub current_direction: (f32, f32),
    /// Strength of the current in pixels per second.
    pub current_strength: f32,
    /// Upward buoyancy force applied to submerged entities.
    pub buoyancy: f32,
    /// Drag factor applied to entity velocity while in water.
    pub drag: f32,
    /// Amplitude of the surface wave animation.
    pub wave_amplitude: f32,
    /// Frequency of the surface wave animation.
    pub wave_frequency: f32,
    /// Internal timer driving wave animation.
    pub wave_timer: f32,
    /// Optional name of the splash particle effect to spawn on entry/exit.
    pub splash_effect: Option<String>,
    /// RGBA tint applied to the screen while the camera is underwater.
    pub underwater_tint: [u8; 4],
}

impl Default for WaterZoneComponent {
    fn default() -> Self {
        Self {
            zone_rect: (0.0, 0.0, 256.0, 128.0),
            surface_y: 0.0,
            current_direction: (0.0, 0.0),
            current_strength: 0.0,
            buoyancy: 200.0,
            drag: 0.85,
            wave_amplitude: 3.0,
            wave_frequency: 2.0,
            wave_timer: 0.0,
            splash_effect: Some("splash_default".to_string()),
            underwater_tint: [0, 40, 80, 60],
        }
    }
}

impl Component for WaterZoneComponent {}
