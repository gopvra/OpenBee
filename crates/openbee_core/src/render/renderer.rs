//! Core rendering trait and basic types.

use glam::Vec2;

use super::sprite::Sprite;

/// RGBA color with each channel in 0..=255.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    /// Create a new color from RGBA values.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB values with full opacity.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a color from a 32-bit hex value (0xRRGGBBAA).
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }
}

/// Axis-aligned rectangle defined by position and size.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check whether a point lies inside the rectangle.
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Check whether two rectangles overlap.
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// Camera parameters for world-to-screen transformation.
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera center position in world coordinates (pixels).
    pub position: Vec2,
    /// Zoom level (1.0 = no zoom, 2.0 = 2x zoom in).
    pub zoom: f32,
    /// Viewport rectangle on screen.
    pub viewport: Rect,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport: Rect::new(0.0, 0.0, 800.0, 600.0),
        }
    }
}

/// Trait for all rendering backends.
pub trait Renderer {
    /// Clear the screen with the given color.
    fn clear(&mut self, color: Color);

    /// Present the back buffer to the screen.
    fn present(&mut self);

    /// Draw a sprite at the given world position with optional source and destination rects.
    fn draw_sprite(
        &mut self,
        sprite: &Sprite,
        position: Vec2,
        src_rect: Option<Rect>,
        flip_x: bool,
        flip_y: bool,
    );

    /// Draw a filled rectangle.
    fn draw_rect_filled(&mut self, rect: Rect, color: Color);

    /// Draw a rectangle outline.
    fn draw_rect_outline(&mut self, rect: Rect, color: Color);

    /// Draw text at the given position.
    fn draw_text(&mut self, text: &str, position: Vec2, size: f32, color: Color);

    /// Set the current camera for world-to-screen transformation.
    fn set_camera(&mut self, camera: &Camera);

    /// Return the current screen (window) size in pixels.
    fn screen_size(&self) -> (u32, u32);
}
