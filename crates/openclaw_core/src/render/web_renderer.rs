//! Web/WASM renderer backend.
//!
//! Provides a renderer implementation for wasm32 targets. When compiled
//! natively this module exposes a stub `WebRenderer` that can be used for
//! testing without a real browser context.

use crate::render::renderer::{Camera, Color, Rect, Renderer};
use crate::render::sprite::Sprite;
use glam::Vec2;
use tracing::info;

/// Renderer implementation for WASM targets.
pub struct WebRenderer {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub pixel_ratio: f32,
    frame_count: u64,
}

impl WebRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        info!("WebRenderer created ({}x{})", width, height);
        Self {
            canvas_width: width,
            canvas_height: height,
            pixel_ratio: 1.0,
            frame_count: 0,
        }
    }
}

impl Renderer for WebRenderer {
    fn clear(&mut self, _color: Color) {
        self.frame_count += 1;
    }

    fn present(&mut self) {}

    fn draw_sprite(
        &mut self,
        _sprite: &Sprite,
        _position: Vec2,
        _src_rect: Option<Rect>,
        _flip_x: bool,
        _flip_y: bool,
    ) {
    }

    fn draw_rect_filled(&mut self, _rect: Rect, _color: Color) {}

    fn draw_rect_outline(&mut self, _rect: Rect, _color: Color) {}

    fn draw_text(&mut self, _text: &str, _position: Vec2, _size: f32, _color: Color) {}

    fn set_camera(&mut self, _camera: &Camera) {}

    fn screen_size(&self) -> (u32, u32) {
        (self.canvas_width, self.canvas_height)
    }
}
