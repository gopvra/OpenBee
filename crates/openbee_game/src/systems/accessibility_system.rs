//! Accessibility system — colorblind modes, difficulty assists, UI scaling.

use openbee_core::ecs::{System, World};
use serde::{Deserialize, Serialize};

/// Maximum allowed size for accessibility config files (1 MB).
const MAX_CONFIG_SIZE: u64 = 1024 * 1024;

/// Colorblind simulation modes.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub enum ColorblindMode {
    #[default]
    None,
    Protanopia,
    Deuteranopia,
    Tritanopia,
    Achromatopsia,
}

/// Subtitle size presets.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SubtitleSize {
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
}

impl SubtitleSize {
    pub fn scale(&self) -> f32 {
        match self {
            Self::Small => 0.75,
            Self::Medium => 1.0,
            Self::Large => 1.5,
            Self::ExtraLarge => 2.0,
        }
    }
}

/// All accessibility settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    // Visual
    pub colorblind_mode: ColorblindMode,
    pub high_contrast: bool,
    pub screen_flash_reduction: bool,
    pub ui_scale: f32,
    pub subtitle_size: SubtitleSize,
    pub show_damage_numbers: bool,
    pub enemy_highlight: bool,
    pub pickup_highlight: bool,

    // Gameplay assists
    pub auto_aim_assist: bool,
    pub extended_coyote_time: f32,
    pub reduced_enemy_damage: f32,
    pub increased_pickup_range: f32,
    pub infinite_lives: bool,
    pub checkpoint_anywhere: bool,
    pub invulnerability_after_hit: f32,
    pub slow_motion_factor: f32,

    // Audio
    pub visual_audio_cues: bool,
    pub mono_audio: bool,
    pub caption_sounds: bool,

    // Input
    pub one_handed_mode: bool,
    pub hold_instead_of_toggle: bool,
    pub input_buffer_window: f32,
    pub auto_run: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            colorblind_mode: ColorblindMode::None,
            high_contrast: false,
            screen_flash_reduction: false,
            ui_scale: 1.0,
            subtitle_size: SubtitleSize::Medium,
            show_damage_numbers: true,
            enemy_highlight: false,
            pickup_highlight: false,
            auto_aim_assist: false,
            extended_coyote_time: 0.0,
            reduced_enemy_damage: 1.0,
            increased_pickup_range: 1.0,
            infinite_lives: false,
            checkpoint_anywhere: false,
            invulnerability_after_hit: 0.0,
            slow_motion_factor: 1.0,
            visual_audio_cues: false,
            mono_audio: false,
            caption_sounds: false,
            one_handed_mode: false,
            hold_instead_of_toggle: false,
            input_buffer_window: 0.1,
            auto_run: false,
        }
    }
}

/// System that applies accessibility transforms to gameplay.
pub struct AccessibilitySystem {
    pub settings: AccessibilitySettings,
}

impl Default for AccessibilitySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilitySystem {
    pub fn new() -> Self {
        Self {
            settings: AccessibilitySettings::default(),
        }
    }

    /// Apply colorblind filter to an RGB color.
    pub fn apply_color_filter(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        match self.settings.colorblind_mode {
            ColorblindMode::None => (r, g, b),
            ColorblindMode::Protanopia => {
                let rf = r as f32 / 255.0;
                let gf = g as f32 / 255.0;
                let bf = b as f32 / 255.0;
                let nr = 0.567 * rf + 0.433 * gf;
                let ng = 0.558 * rf + 0.442 * gf;
                let nb = 0.242 * gf + 0.758 * bf;
                (
                    (nr.clamp(0.0, 1.0) * 255.0) as u8,
                    (ng.clamp(0.0, 1.0) * 255.0) as u8,
                    (nb.clamp(0.0, 1.0) * 255.0) as u8,
                )
            }
            ColorblindMode::Deuteranopia => {
                let rf = r as f32 / 255.0;
                let gf = g as f32 / 255.0;
                let bf = b as f32 / 255.0;
                let nr = 0.625 * rf + 0.375 * gf;
                let ng = 0.7 * rf + 0.3 * gf;
                let nb = 0.3 * gf + 0.7 * bf;
                (
                    (nr.clamp(0.0, 1.0) * 255.0) as u8,
                    (ng.clamp(0.0, 1.0) * 255.0) as u8,
                    (nb.clamp(0.0, 1.0) * 255.0) as u8,
                )
            }
            ColorblindMode::Tritanopia => {
                let rf = r as f32 / 255.0;
                let gf = g as f32 / 255.0;
                let bf = b as f32 / 255.0;
                let nr = 0.95 * rf + 0.05 * gf;
                let ng = 0.433 * gf + 0.567 * bf;
                let nb = 0.475 * gf + 0.525 * bf;
                (
                    (nr.clamp(0.0, 1.0) * 255.0) as u8,
                    (ng.clamp(0.0, 1.0) * 255.0) as u8,
                    (nb.clamp(0.0, 1.0) * 255.0) as u8,
                )
            }
            ColorblindMode::Achromatopsia => {
                let gray = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
                (gray, gray, gray)
            }
        }
    }

    pub fn get_coyote_time(&self, base: f32) -> f32 {
        base + self.settings.extended_coyote_time
    }

    pub fn get_damage_multiplier(&self) -> f32 {
        self.settings.reduced_enemy_damage
    }

    pub fn get_pickup_range(&self, base: f32) -> f32 {
        base * self.settings.increased_pickup_range
    }

    pub fn get_iframes(&self, base: f32) -> f32 {
        base + self.settings.invulnerability_after_hit
    }

    pub fn get_ui_scale(&self) -> f32 {
        self.settings.ui_scale
    }

    pub fn should_show_flash(&self) -> bool {
        !self.settings.screen_flash_reduction
    }

    // NOTE: Caller is responsible for sandbox validation
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.settings).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    // NOTE: Caller is responsible for sandbox validation
    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        // Validate file size to prevent memory exhaustion attacks
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_CONFIG_SIZE {
            return Err(std::io::Error::other(format!(
                "File too large: {} bytes (max {})",
                metadata.len(),
                MAX_CONFIG_SIZE
            )));
        }

        let json = std::fs::read_to_string(path)?;
        self.settings = serde_json::from_str(&json).map_err(std::io::Error::other)?;
        Ok(())
    }
}

impl System for AccessibilitySystem {
    fn name(&self) -> &str {
        "AccessibilitySystem"
    }

    fn update(&mut self, _world: &mut World, _dt: f32) {
        // Accessibility settings are read by other systems;
        // this system exists for lifecycle management and future
        // per-frame accessibility processing (e.g., caption timing).
    }
}
