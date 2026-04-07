//! Shader effect definitions for visual post-processing on sprites.

use std::collections::HashMap;

/// Predefined shader effects for actor rendering.
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderEffect {
    /// No shader effect.
    None,
    /// Flash the sprite white (e.g., on damage).
    Flash,
    /// Apply a glow outline effect.
    Glow,
    /// Invincibility flicker/transparency effect.
    Invincibility,
    /// Custom named shader for modding support.
    Custom(String),
}

/// Parameters that can be passed to shader effects.
#[derive(Debug, Clone)]
pub struct ShaderParams {
    /// Named float parameters.
    pub floats: HashMap<String, f32>,
    /// Named integer parameters.
    pub ints: HashMap<String, i32>,
    /// Named color parameters as [r, g, b, a] (0..=255).
    pub colors: HashMap<String, [u8; 4]>,
}

impl ShaderParams {
    /// Create empty shader parameters.
    pub fn new() -> Self {
        Self {
            floats: HashMap::new(),
            ints: HashMap::new(),
            colors: HashMap::new(),
        }
    }

    /// Set a float parameter.
    pub fn set_float(&mut self, name: impl Into<String>, value: f32) -> &mut Self {
        self.floats.insert(name.into(), value);
        self
    }

    /// Set an integer parameter.
    pub fn set_int(&mut self, name: impl Into<String>, value: i32) -> &mut Self {
        self.ints.insert(name.into(), value);
        self
    }

    /// Set a color parameter.
    pub fn set_color(&mut self, name: impl Into<String>, rgba: [u8; 4]) -> &mut Self {
        self.colors.insert(name.into(), rgba);
        self
    }

    /// Get a float parameter.
    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.floats.get(name).copied()
    }

    /// Get an integer parameter.
    pub fn get_int(&self, name: &str) -> Option<i32> {
        self.ints.get(name).copied()
    }

    /// Get a color parameter.
    pub fn get_color(&self, name: &str) -> Option<[u8; 4]> {
        self.colors.get(name).copied()
    }
}

impl Default for ShaderParams {
    fn default() -> Self {
        Self::new()
    }
}
