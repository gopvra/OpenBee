//! Weather system - rain, snow, fog, wind, thunder effects.

use openclaw_core::ecs::{System, World};
use serde::{Deserialize, Serialize};

/// Types of weather effects.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum WeatherType {
    Clear,
    Rain,
    HeavyRain,
    Snow,
    Fog,
    Sandstorm,
    Thunder,
    Volcanic,
}

impl Default for WeatherType {
    fn default() -> Self {
        Self::Clear
    }
}

/// Configuration for a weather effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    pub weather_type: WeatherType,
    pub intensity: f32,
    pub wind_direction: f32,
    pub wind_strength: f32,
    pub particle_count: u32,
    pub particle_speed: f32,
    pub particle_color: [u8; 4],
    pub fog_density: f32,
    pub fog_color: [u8; 4],
    pub lightning_interval: f32,
    pub ambient_sound: Option<String>,
    pub affects_physics: bool,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            weather_type: WeatherType::Clear,
            intensity: 0.0,
            wind_direction: 0.0,
            wind_strength: 0.0,
            particle_count: 0,
            particle_speed: 200.0,
            particle_color: [200, 200, 255, 180],
            fog_density: 0.0,
            fog_color: [180, 180, 180, 128],
            lightning_interval: 0.0,
            ambient_sound: None,
            affects_physics: false,
        }
    }
}

impl WeatherConfig {
    pub fn rain() -> Self {
        Self {
            weather_type: WeatherType::Rain,
            intensity: 0.7,
            wind_direction: -0.2,
            wind_strength: 30.0,
            particle_count: 500,
            particle_speed: 400.0,
            particle_color: [150, 180, 255, 140],
            ..Default::default()
        }
    }

    pub fn snow() -> Self {
        Self {
            weather_type: WeatherType::Snow,
            intensity: 0.5,
            wind_direction: 0.3,
            wind_strength: 15.0,
            particle_count: 300,
            particle_speed: 60.0,
            particle_color: [255, 255, 255, 200],
            ..Default::default()
        }
    }

    pub fn fog() -> Self {
        Self {
            weather_type: WeatherType::Fog,
            intensity: 0.6,
            fog_density: 0.4,
            fog_color: [180, 180, 200, 160],
            ..Default::default()
        }
    }

    pub fn thunder() -> Self {
        Self {
            weather_type: WeatherType::Thunder,
            intensity: 0.9,
            wind_direction: -0.3,
            wind_strength: 50.0,
            particle_count: 800,
            particle_speed: 500.0,
            particle_color: [140, 160, 255, 160],
            lightning_interval: 5.0,
            ..Default::default()
        }
    }
}

/// Individual weather particle (raindrop, snowflake, sand grain).
#[derive(Debug, Clone)]
pub struct WeatherParticle {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub life: f32,
    pub size: f32,
}

/// Manages weather rendering and effects.
pub struct WeatherSystem {
    pub config: WeatherConfig,
    pub particles: Vec<WeatherParticle>,
    pub active: bool,
    pub transition_timer: f32,
    pub transition_duration: f32,
    pub transition_from: Option<WeatherConfig>,
    pub lightning_timer: f32,
    pub is_lightning_flash: bool,
    pub flash_timer: f32,
    pub wind_noise_offset: f32,
}

impl WeatherSystem {
    pub fn new() -> Self {
        Self {
            config: WeatherConfig::default(),
            particles: Vec::new(),
            active: false,
            transition_timer: 0.0,
            transition_duration: 0.0,
            transition_from: None,
            lightning_timer: 0.0,
            is_lightning_flash: false,
            flash_timer: 0.0,
            wind_noise_offset: 0.0,
        }
    }

    pub fn set_weather(&mut self, config: WeatherConfig, transition_time: f32) {
        if config.weather_type == WeatherType::Clear && self.config.weather_type == WeatherType::Clear {
            return;
        }
        self.transition_from = Some(self.config.clone());
        self.config = config;
        self.transition_timer = 0.0;
        self.transition_duration = transition_time;
        self.active = self.config.weather_type != WeatherType::Clear;
    }

    pub fn clear_weather(&mut self, transition_time: f32) {
        self.set_weather(WeatherConfig::default(), transition_time);
    }

    pub fn update(&mut self, dt: f32, viewport: (f32, f32, f32, f32)) {
        // Transition blending
        if self.transition_duration > 0.0 {
            self.transition_timer += dt;
            if self.transition_timer >= self.transition_duration {
                self.transition_from = None;
                self.transition_duration = 0.0;
            }
        }

        if !self.active && self.particles.is_empty() {
            return;
        }

        let (vx, vy, vw, vh) = viewport;

        // Spawn new particles
        let spawn_count = (self.config.particle_count as f32 * dt).ceil() as u32;
        for _ in 0..spawn_count {
            let px = vx + pseudo_random(self.wind_noise_offset) * vw;
            let py = vy - 20.0;
            self.particles.push(WeatherParticle {
                x: px,
                y: py,
                vel_x: self.config.wind_strength * self.config.wind_direction,
                vel_y: self.config.particle_speed,
                life: 1.0,
                size: 2.0 + pseudo_random(self.wind_noise_offset + 1.0) * 3.0,
            });
            self.wind_noise_offset += 0.1;
        }

        // Update particles
        for p in &mut self.particles {
            p.x += p.vel_x * dt;
            p.y += p.vel_y * dt;
            p.life -= dt * 0.5;
        }

        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0 && p.y < vy + vh + 50.0);

        // Lightning
        if self.config.lightning_interval > 0.0 {
            self.lightning_timer += dt;
            if self.lightning_timer >= self.config.lightning_interval {
                self.lightning_timer = 0.0;
                self.is_lightning_flash = true;
                self.flash_timer = 0.15;
            }
        }
        if self.is_lightning_flash {
            self.flash_timer -= dt;
            if self.flash_timer <= 0.0 {
                self.is_lightning_flash = false;
            }
        }
    }

    pub fn get_wind_force(&self) -> (f32, f32) {
        if self.config.affects_physics {
            (
                self.config.wind_strength * self.config.wind_direction,
                0.0,
            )
        } else {
            (0.0, 0.0)
        }
    }

    pub fn is_lightning(&self) -> bool {
        self.is_lightning_flash
    }

    pub fn get_fog_alpha(&self) -> f32 {
        self.config.fog_density * self.config.intensity
    }
}

fn pseudo_random(seed: f32) -> f32 {
    let x = (seed * 12.9898).sin() * 43758.5453;
    x - x.floor()
}

impl System for WeatherSystem {
    fn name(&self) -> &str {
        "WeatherSystem"
    }

    fn update(&mut self, _world: &mut World, dt: f32) {
        self.update(dt, (0.0, 0.0, 1280.0, 720.0));
    }
}
