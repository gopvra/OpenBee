//! Rendering abstractions for sprites, animations, particles, and shaders.

pub mod animation;
pub mod particle;
pub mod renderer;
pub mod screenshot;
pub mod sdl2_renderer;
pub mod shader;
pub mod sprite;
pub mod web_renderer;

pub use animation::{Animation, AnimationPlayer};
pub use particle::{Particle, ParticleEmitter, ParticleSystem};
pub use renderer::{Camera, Color, Rect, Renderer};
pub use sdl2_renderer::NullRenderer;
pub use shader::{ShaderEffect, ShaderParams};
pub use sprite::{Sprite, SpriteId, SpriteSheet};
