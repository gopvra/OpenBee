//! SDL2 renderer placeholder and NullRenderer for headless/testing use.
//!
//! The `Sdl2Renderer` struct is a placeholder that would require the `sdl2` feature
//! and the actual SDL2 library bindings. For now, only `NullRenderer` is fully implemented.

use glam::Vec2;

use super::renderer::{Camera, Color, Rect, Renderer};
use super::sprite::Sprite;

/// Placeholder struct for a future SDL2-based renderer.
/// Requires the `sdl2` crate and feature flag to be functional.
#[allow(dead_code)]
pub struct Sdl2Renderer {
    _private: (),
}

/// A no-op renderer that logs calls via tracing. Useful for headless testing
/// or running the engine without a display.
pub struct NullRenderer {
    screen_width: u32,
    screen_height: u32,
    camera: Camera,
}

impl NullRenderer {
    /// Create a new null renderer with the given virtual screen size.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            screen_width: width,
            screen_height: height,
            camera: Camera::default(),
        }
    }
}

impl Renderer for NullRenderer {
    fn clear(&mut self, color: Color) {
        tracing::trace!(
            "NullRenderer::clear({}, {}, {}, {})",
            color.r,
            color.g,
            color.b,
            color.a
        );
    }

    fn present(&mut self) {
        tracing::trace!("NullRenderer::present()");
    }

    fn draw_sprite(
        &mut self,
        _sprite: &Sprite,
        position: Vec2,
        _src_rect: Option<Rect>,
        flip_x: bool,
        flip_y: bool,
    ) {
        tracing::trace!(
            "NullRenderer::draw_sprite(pos=({}, {}), flip=({}, {}))",
            position.x,
            position.y,
            flip_x,
            flip_y
        );
    }

    fn draw_rect_filled(&mut self, rect: Rect, _color: Color) {
        tracing::trace!(
            "NullRenderer::draw_rect_filled({}, {}, {}, {})",
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );
    }

    fn draw_rect_outline(&mut self, rect: Rect, _color: Color) {
        tracing::trace!(
            "NullRenderer::draw_rect_outline({}, {}, {}, {})",
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );
    }

    fn draw_text(&mut self, text: &str, position: Vec2, size: f32, _color: Color) {
        tracing::trace!(
            "NullRenderer::draw_text(\"{}\", ({}, {}), size={})",
            text,
            position.x,
            position.y,
            size
        );
    }

    fn set_camera(&mut self, camera: &Camera) {
        self.camera = camera.clone();
    }

    fn screen_size(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }
}
