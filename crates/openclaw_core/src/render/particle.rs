//! Particle system for visual effects.

use glam::Vec2;

use super::renderer::Color;

/// A single particle in the system.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Position in world-space pixels.
    pub position: Vec2,
    /// Velocity in pixels per second.
    pub velocity: Vec2,
    /// Remaining lifetime in seconds.
    pub life: f32,
    /// Maximum lifetime in seconds (used for fade calculations).
    pub max_life: f32,
    /// Particle color.
    pub color: Color,
    /// Particle size in pixels.
    pub size: f32,
}

impl Particle {
    /// Returns the normalized life remaining (1.0 = just born, 0.0 = about to die).
    pub fn life_ratio(&self) -> f32 {
        if self.max_life <= 0.0 {
            0.0
        } else {
            (self.life / self.max_life).clamp(0.0, 1.0)
        }
    }

    /// Check if the particle is dead.
    pub fn is_dead(&self) -> bool {
        self.life <= 0.0
    }
}

/// Configuration for how particles are emitted.
#[derive(Debug, Clone)]
pub struct ParticleEmitterConfig {
    /// Particles emitted per second.
    pub rate: f32,
    /// Particle lifetime in seconds.
    pub lifetime: f32,
    /// Random lifetime variance (+/- seconds).
    pub lifetime_variance: f32,
    /// Spread angle in radians (0 = straight, PI = hemisphere, 2*PI = full circle).
    pub spread: f32,
    /// Base emission direction angle in radians.
    pub direction: f32,
    /// Base speed in pixels/second.
    pub speed: f32,
    /// Random speed variance.
    pub speed_variance: f32,
    /// Start color.
    pub color_start: Color,
    /// End color (for lerping over lifetime).
    pub color_end: Color,
    /// Start size in pixels.
    pub size_start: f32,
    /// End size in pixels.
    pub size_end: f32,
    /// Gravity applied to particles (pixels/s^2).
    pub gravity: Vec2,
}

impl Default for ParticleEmitterConfig {
    fn default() -> Self {
        Self {
            rate: 10.0,
            lifetime: 1.0,
            lifetime_variance: 0.2,
            spread: std::f32::consts::PI * 0.25,
            direction: -std::f32::consts::FRAC_PI_2, // upward
            speed: 50.0,
            speed_variance: 10.0,
            color_start: Color::WHITE,
            color_end: Color::TRANSPARENT,
            size_start: 4.0,
            size_end: 1.0,
            gravity: Vec2::new(0.0, 98.0),
        }
    }
}

/// Emits particles at a configurable rate from a given position.
pub struct ParticleEmitter {
    /// Emitter configuration.
    pub config: ParticleEmitterConfig,
    /// Position in world-space pixels.
    pub position: Vec2,
    /// Whether the emitter is currently active.
    pub active: bool,
    /// Accumulated time for emission rate.
    emission_accumulator: f32,
    /// Simple pseudo-random state.
    rng_state: u32,
}

impl ParticleEmitter {
    /// Create a new emitter at the given position.
    pub fn new(position: Vec2, config: ParticleEmitterConfig) -> Self {
        Self {
            config,
            position,
            active: true,
            emission_accumulator: 0.0,
            rng_state: 12345,
        }
    }

    /// Spawn a single particle based on the emitter's configuration.
    pub fn spawn(&mut self) -> Particle {
        let angle_offset = self.random_range(-self.config.spread * 0.5, self.config.spread * 0.5);
        let angle = self.config.direction + angle_offset;
        let speed =
            self.config.speed + self.random_range(-self.config.speed_variance, self.config.speed_variance);
        let lifetime = (self.config.lifetime
            + self.random_range(-self.config.lifetime_variance, self.config.lifetime_variance))
            .max(0.01);

        Particle {
            position: self.position,
            velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            life: lifetime,
            max_life: lifetime,
            color: self.config.color_start,
            size: self.config.size_start,
        }
    }

    /// Simple xorshift pseudo-random number generator.
    fn next_random(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as f32) / (u32::MAX as f32)
    }

    /// Generate a random value in the given range.
    fn random_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_random() * (max - min)
    }
}

/// Manages a collection of particles and their emitters.
pub struct ParticleSystem {
    /// All active particles.
    pub particles: Vec<Particle>,
    /// All emitters in the system.
    pub emitters: Vec<ParticleEmitter>,
    /// Maximum number of particles allowed.
    pub max_particles: usize,
}

impl ParticleSystem {
    /// Create a new particle system with the given capacity.
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            emitters: Vec::new(),
            max_particles,
        }
    }

    /// Add an emitter to the system.
    pub fn add_emitter(&mut self, emitter: ParticleEmitter) {
        self.emitters.push(emitter);
    }

    /// Update all particles and emitters by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        // Update existing particles.
        for particle in &mut self.particles {
            particle.life -= dt;
            particle.velocity += Vec2::ZERO; // gravity is per-emitter; applied at spawn
            particle.position += particle.velocity * dt;
        }

        // Remove dead particles.
        self.particles.retain(|p| !p.is_dead());

        // Spawn new particles from active emitters.
        // We collect spawns to avoid borrow issues.
        let mut new_particles = Vec::new();
        for emitter in &mut self.emitters {
            if !emitter.active {
                continue;
            }
            emitter.emission_accumulator += dt;
            let interval = 1.0 / emitter.config.rate;
            while emitter.emission_accumulator >= interval {
                emitter.emission_accumulator -= interval;
                if self.particles.len() + new_particles.len() < self.max_particles {
                    new_particles.push(emitter.spawn());
                }
            }
        }
        self.particles.extend(new_particles);
    }

    /// Update particles with per-emitter gravity. Call this instead of `update` if you want
    /// gravity from emitter configs to be applied.
    pub fn update_with_gravity(&mut self, dt: f32) {
        // Apply gravity from the first emitter config for simplicity.
        let gravity = self
            .emitters
            .first()
            .map(|e| e.config.gravity)
            .unwrap_or(Vec2::ZERO);

        for particle in &mut self.particles {
            particle.life -= dt;
            particle.velocity += gravity * dt;
            particle.position += particle.velocity * dt;
        }

        self.particles.retain(|p| !p.is_dead());

        let mut new_particles = Vec::new();
        for emitter in &mut self.emitters {
            if !emitter.active {
                continue;
            }
            emitter.emission_accumulator += dt;
            let interval = 1.0 / emitter.config.rate;
            while emitter.emission_accumulator >= interval {
                emitter.emission_accumulator -= interval;
                if self.particles.len() + new_particles.len() < self.max_particles {
                    new_particles.push(emitter.spawn());
                }
            }
        }
        self.particles.extend(new_particles);
    }

    /// Return the number of active particles.
    pub fn active_count(&self) -> usize {
        self.particles.len()
    }

    /// Clear all particles (but keep emitters).
    pub fn clear_particles(&mut self) {
        self.particles.clear();
    }

    /// Clear everything including emitters.
    pub fn clear_all(&mut self) {
        self.particles.clear();
        self.emitters.clear();
    }
}
