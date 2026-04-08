//! Game juice/feel system for visual feedback effects.
//!
//! Inspired by Celeste and Hollow Knight, provides screen shake, hit freeze,
//! squash & stretch, ghost trails, hit flash, and slow motion.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Configuration for game feel effects.
#[derive(Debug, Clone)]
pub struct JuiceConfig {
    pub screen_shake_enabled: bool,
    pub hit_freeze_enabled: bool,
    pub squash_stretch_enabled: bool,
    pub trail_enabled: bool,
}

impl Default for JuiceConfig {
    fn default() -> Self {
        Self {
            screen_shake_enabled: true,
            hit_freeze_enabled: true,
            squash_stretch_enabled: true,
            trail_enabled: true,
        }
    }
}

/// Screen freeze/hitlag effect — freezes game for N frames on impact.
#[derive(Debug, Clone)]
pub struct HitFreeze {
    pub remaining_frames: u32,
    pub time_scale: f32,
}

/// Squash and stretch deformation applied to sprites.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquashStretch {
    pub scale_x: f32,
    pub scale_y: f32,
    pub recovery_speed: f32,
}

impl SquashStretch {
    /// Create a new squash-stretch at rest state.
    pub fn new() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            recovery_speed: 8.0,
        }
    }

    /// Stretch vertically on jump (tall and thin).
    pub fn apply_jump(&mut self) {
        self.scale_x = 0.7;
        self.scale_y = 1.4;
    }

    /// Squash on landing (wide and flat).
    pub fn apply_land(&mut self) {
        self.scale_x = 1.4;
        self.scale_y = 0.6;
    }

    /// Squash on taking damage (compressed inward).
    pub fn apply_hit(&mut self) {
        self.scale_x = 1.3;
        self.scale_y = 0.7;
    }

    /// Bounce effect (alternating squash).
    pub fn apply_bounce(&mut self) {
        self.scale_x = 0.8;
        self.scale_y = 1.3;
    }

    /// Update toward rest state. Returns current (scale_x, scale_y).
    pub fn update(&mut self, dt: f32) -> (f32, f32) {
        let speed = self.recovery_speed * dt;
        self.scale_x += (1.0 - self.scale_x) * speed;
        self.scale_y += (1.0 - self.scale_y) * speed;

        // Snap to 1.0 when very close to avoid perpetual micro-oscillation.
        if (self.scale_x - 1.0).abs() < 0.001 {
            self.scale_x = 1.0;
        }
        if (self.scale_y - 1.0).abs() < 0.001 {
            self.scale_y = 1.0;
        }

        (self.scale_x, self.scale_y)
    }
}

impl Default for SquashStretch {
    fn default() -> Self {
        Self::new()
    }
}

/// A single ghost in a trail effect.
#[derive(Debug, Clone)]
pub struct TrailGhost {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub opacity: f32,
    pub lifetime: f32,
    pub sprite_frame: u32,
}

/// Ghost/afterimage trail for fast-moving entities.
#[derive(Debug, Clone)]
pub struct TrailEffect {
    pub ghosts: Vec<TrailGhost>,
    pub spawn_interval: f32,
    pub spawn_timer: f32,
    pub max_ghosts: usize,
    pub ghost_lifetime: f32,
    pub color_tint: [u8; 4],
    pub opacity_decay: f32,
}

impl TrailEffect {
    /// Create a new trail effect.
    pub fn new(max_ghosts: usize, interval: f32, lifetime: f32) -> Self {
        Self {
            ghosts: Vec::with_capacity(max_ghosts),
            spawn_interval: interval,
            spawn_timer: 0.0,
            max_ghosts,
            ghost_lifetime: lifetime,
            color_tint: [255, 255, 255, 180],
            opacity_decay: 1.0,
        }
    }

    /// Tick the trail. Spawns new ghosts and fades/removes old ones.
    pub fn update(
        &mut self,
        dt: f32,
        current_pos: Vec2,
        current_scale: Vec2,
        rotation: f32,
        frame: u32,
    ) {
        // Age existing ghosts.
        for ghost in &mut self.ghosts {
            ghost.lifetime -= dt;
            if self.ghost_lifetime > 0.0 {
                ghost.opacity =
                    (ghost.lifetime / self.ghost_lifetime).max(0.0) * self.opacity_decay;
            }
        }
        // Remove dead ghosts.
        self.ghosts.retain(|g| g.lifetime > 0.0);

        // Try to spawn a new ghost.
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer = self.spawn_interval;

            if self.ghosts.len() < self.max_ghosts {
                self.ghosts.push(TrailGhost {
                    position: current_pos,
                    scale: current_scale,
                    rotation,
                    opacity: 1.0,
                    lifetime: self.ghost_lifetime,
                    sprite_frame: frame,
                });
            }
        }
    }

    /// Remove all active ghosts.
    pub fn clear(&mut self) {
        self.ghosts.clear();
        self.spawn_timer = 0.0;
    }
}

/// Flash white/color on hit.
#[derive(Debug, Clone)]
pub struct HitFlash {
    pub color: [u8; 4],
    pub duration: f32,
    pub remaining: f32,
    pub active: bool,
}

impl HitFlash {
    /// Start a new flash.
    pub fn trigger(&mut self, color: [u8; 4], duration: f32) {
        self.color = color;
        self.duration = duration;
        self.remaining = duration;
        self.active = true;
    }

    /// Tick the flash timer. Returns `true` while the flash is active.
    pub fn update(&mut self, dt: f32) -> bool {
        if !self.active {
            return false;
        }
        self.remaining -= dt;
        if self.remaining <= 0.0 {
            self.remaining = 0.0;
            self.active = false;
        }
        self.active
    }
}

impl Default for HitFlash {
    fn default() -> Self {
        Self {
            color: [255, 255, 255, 255],
            duration: 0.0,
            remaining: 0.0,
            active: false,
        }
    }
}

/// Slow motion ("bullet time") effect.
#[derive(Debug, Clone)]
pub struct SlowMotion {
    pub time_scale: f32,
    pub duration: f32,
    pub remaining: f32,
    pub ease_in: bool,
    pub ease_out: bool,
}

/// Master juice manager that coordinates global effects.
pub struct JuiceManager {
    pub config: JuiceConfig,
    pub hit_freeze: Option<HitFreeze>,
    pub slow_motion: Option<SlowMotion>,
    pub global_time_scale: f32,
    pub screen_flash: Option<([u8; 4], f32)>,
}

impl JuiceManager {
    /// Create a new juice manager with default configuration.
    pub fn new() -> Self {
        Self {
            config: JuiceConfig::default(),
            hit_freeze: None,
            slow_motion: None,
            global_time_scale: 1.0,
            screen_flash: None,
        }
    }

    /// Celeste-style hit stop: freeze for a number of frames.
    pub fn freeze_frame(&mut self, frames: u32) {
        if !self.config.hit_freeze_enabled {
            return;
        }
        self.hit_freeze = Some(HitFreeze {
            remaining_frames: frames,
            time_scale: 0.0,
        });
    }

    /// Freeze time at a custom time scale for a given duration (seconds).
    pub fn freeze_time(&mut self, duration: f32, scale: f32) {
        if !self.config.hit_freeze_enabled {
            return;
        }
        // Convert duration to an approximate frame count at 60 fps so we can
        // reuse the HitFreeze struct.  The actual countdown happens in `update`
        // via frame ticks.
        let frames = (duration * 60.0).ceil() as u32;
        self.hit_freeze = Some(HitFreeze {
            remaining_frames: frames,
            time_scale: scale.clamp(0.0, 1.0),
        });
    }

    /// Activate slow motion with optional ease in/out.
    pub fn slow_motion(&mut self, scale: f32, duration: f32, ease_in: bool, ease_out: bool) {
        self.slow_motion = Some(SlowMotion {
            time_scale: scale.clamp(0.0, 1.0),
            duration,
            remaining: duration,
            ease_in,
            ease_out,
        });
    }

    /// Flash the entire screen with a color for a duration.
    pub fn flash_screen(&mut self, color: [u8; 4], duration: f32) {
        self.screen_flash = Some((color, duration));
    }

    /// Tick the manager. Returns the effective time scale for the current frame.
    pub fn update(&mut self, dt: f32) -> f32 {
        let mut scale = self.global_time_scale;

        // --- Hit freeze ---
        if let Some(ref mut freeze) = self.hit_freeze {
            if freeze.remaining_frames > 0 {
                freeze.remaining_frames -= 1;
                scale *= freeze.time_scale;
            }
            if freeze.remaining_frames == 0 {
                self.hit_freeze = None;
            }
        }

        // --- Slow motion ---
        if let Some(ref mut sm) = self.slow_motion {
            sm.remaining -= dt;

            let progress = if sm.duration > 0.0 {
                1.0 - (sm.remaining / sm.duration).clamp(0.0, 1.0)
            } else {
                1.0
            };

            // Compute the current slow-mo factor with optional ease in/out.
            let factor = if sm.ease_in && progress < 0.3 {
                // Ease in during the first 30%.
                let t = progress / 0.3;
                let eased = t * t; // quadratic ease-in
                1.0 - (1.0 - sm.time_scale) * eased
            } else if sm.ease_out && progress > 0.7 {
                // Ease out during the last 30%.
                let t = (progress - 0.7) / 0.3;
                let eased = t * t; // quadratic ease-out (inverted)
                sm.time_scale + (1.0 - sm.time_scale) * eased
            } else {
                sm.time_scale
            };

            scale *= factor;

            if sm.remaining <= 0.0 {
                self.slow_motion = None;
            }
        }

        // --- Screen flash ---
        if let Some((_, ref mut remaining)) = self.screen_flash {
            *remaining -= dt;
            if *remaining <= 0.0 {
                self.screen_flash = None;
            }
        }

        scale.max(0.0)
    }

    /// Returns `true` when a hit freeze is actively pausing the game.
    pub fn is_frozen(&self) -> bool {
        self.hit_freeze
            .as_ref()
            .is_some_and(|f| f.remaining_frames > 0 && f.time_scale == 0.0)
    }

    /// Compute the effective delta time after applying all active juice effects.
    pub fn effective_dt(&self, dt: f32) -> f32 {
        let mut scale = self.global_time_scale;

        if let Some(ref freeze) = self.hit_freeze {
            if freeze.remaining_frames > 0 {
                scale *= freeze.time_scale;
            }
        }

        if let Some(ref sm) = self.slow_motion {
            if sm.remaining > 0.0 {
                scale *= sm.time_scale;
            }
        }

        (dt * scale).max(0.0)
    }
}

impl Default for JuiceManager {
    fn default() -> Self {
        Self::new()
    }
}
