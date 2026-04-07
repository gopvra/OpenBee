//! Sprite and sprite sheet types.

use glam::Vec2;
use std::collections::HashMap;

/// Unique identifier for a sprite within a sprite sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteId(pub u32);

/// A single sprite image stored as raw RGBA pixel data.
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Raw RGBA pixel data (4 bytes per pixel, row-major).
    pub pixels: Vec<u8>,
    /// Draw offset relative to the entity's position (used for sprite alignment).
    pub offset: Vec2,
}

impl Sprite {
    /// Create a new sprite with the given dimensions and pixel data.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>, offset: Vec2) -> Self {
        debug_assert_eq!(
            pixels.len(),
            (width * height * 4) as usize,
            "Pixel data length must match width * height * 4"
        );
        Self {
            width,
            height,
            pixels,
            offset,
        }
    }

    /// Create an empty (transparent) sprite with the given dimensions.
    pub fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0u8; (width * height * 4) as usize],
            offset: Vec2::ZERO,
        }
    }

    /// Get the RGBA color of a pixel at (x, y).
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }

    /// Set the RGBA color of a pixel at (x, y).
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        let idx = ((y * self.width + x) * 4) as usize;
        self.pixels[idx] = rgba[0];
        self.pixels[idx + 1] = rgba[1];
        self.pixels[idx + 2] = rgba[2];
        self.pixels[idx + 3] = rgba[3];
    }
}

/// A collection of sprites accessible by name or by index.
pub struct SpriteSheet {
    sprites: Vec<Sprite>,
    name_to_index: HashMap<String, usize>,
}

impl SpriteSheet {
    /// Create a new empty sprite sheet.
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    /// Add a sprite with an optional name. Returns the sprite's index.
    pub fn add_sprite(&mut self, name: Option<&str>, sprite: Sprite) -> SpriteId {
        let index = self.sprites.len();
        self.sprites.push(sprite);
        if let Some(name) = name {
            self.name_to_index.insert(name.to_string(), index);
        }
        SpriteId(index as u32)
    }

    /// Get a sprite by its ID.
    pub fn get(&self, id: SpriteId) -> Option<&Sprite> {
        self.sprites.get(id.0 as usize)
    }

    /// Get a sprite by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Sprite> {
        self.name_to_index
            .get(name)
            .and_then(|&idx| self.sprites.get(idx))
    }

    /// Look up a sprite ID by name.
    pub fn id_by_name(&self, name: &str) -> Option<SpriteId> {
        self.name_to_index
            .get(name)
            .map(|&idx| SpriteId(idx as u32))
    }

    /// Return the total number of sprites.
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    /// Check if the sprite sheet is empty.
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }
}

impl Default for SpriteSheet {
    fn default() -> Self {
        Self::new()
    }
}
