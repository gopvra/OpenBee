//! Tile map plane node for rendering tile-based levels with parallax scrolling.

use glam::Vec2;

use crate::render::renderer::{Camera, Color, Rect, Renderer};

use super::node::{SceneNode, SceneNodeType};

/// A single tile layer containing a grid of tile IDs.
#[derive(Debug, Clone)]
pub struct TileLayer {
    /// Width of the tile grid in tiles.
    pub width: u32,
    /// Height of the tile grid in tiles.
    pub height: u32,
    /// Size of each tile in pixels.
    pub tile_size: u32,
    /// Tile data: `None` means empty/transparent, `Some(id)` references a tile in the tileset.
    pub tiles: Vec<Option<u32>>,
}

impl TileLayer {
    /// Create a new empty tile layer.
    pub fn new(width: u32, height: u32, tile_size: u32) -> Self {
        let count = (width * height) as usize;
        Self {
            width,
            height,
            tile_size,
            tiles: vec![None; count],
        }
    }

    /// Get the tile at grid coordinates (tx, ty).
    pub fn get_tile(&self, tx: u32, ty: u32) -> Option<u32> {
        if tx >= self.width || ty >= self.height {
            return None;
        }
        self.tiles[(ty * self.width + tx) as usize]
    }

    /// Set the tile at grid coordinates (tx, ty).
    pub fn set_tile(&mut self, tx: u32, ty: u32, tile_id: Option<u32>) {
        if tx < self.width && ty < self.height {
            self.tiles[(ty * self.width + tx) as usize] = tile_id;
        }
    }

    /// Total width of the layer in pixels.
    pub fn pixel_width(&self) -> u32 {
        self.width * self.tile_size
    }

    /// Total height of the layer in pixels.
    pub fn pixel_height(&self) -> u32 {
        self.height * self.tile_size
    }
}

/// Scene node that renders a tile map layer with optional parallax scrolling.
pub struct TilePlaneNode {
    /// The tile layer data.
    pub layer: TileLayer,
    /// Z-order for draw sorting.
    pub z: i32,
    /// Whether this layer is visible.
    pub visible: bool,
    /// Parallax scroll factor (1.0 = normal, 0.5 = half-speed, 0.0 = fixed background).
    pub parallax_factor: Vec2,
    /// Optional fixed offset in pixels.
    pub offset: Vec2,
    /// Whether the layer tiles horizontally.
    pub wrap_x: bool,
    /// Whether the layer tiles vertically.
    pub wrap_y: bool,
}

impl TilePlaneNode {
    /// Create a new tile plane node.
    pub fn new(layer: TileLayer, z_order: i32, parallax_factor: Vec2) -> Self {
        Self {
            layer,
            z: z_order,
            visible: true,
            parallax_factor,
            offset: Vec2::ZERO,
            wrap_x: false,
            wrap_y: false,
        }
    }
}

impl SceneNode for TilePlaneNode {
    fn update(&mut self, _dt: f32) {
        // Tile layers are static; animation of individual tiles would be handled separately.
    }

    fn render(&self, renderer: &mut dyn Renderer, camera: &Camera) {
        if !self.visible || self.layer.tile_size == 0 {
            return;
        }

        let ts = self.layer.tile_size as f32;

        // Calculate the effective camera offset with parallax.
        let cam_offset = Vec2::new(
            camera.position.x * self.parallax_factor.x,
            camera.position.y * self.parallax_factor.y,
        ) + self.offset;

        // Determine the visible tile range.
        let vp = &camera.viewport;
        let start_x = ((cam_offset.x - vp.width * 0.5) / ts).floor() as i32;
        let start_y = ((cam_offset.y - vp.height * 0.5) / ts).floor() as i32;
        let end_x = ((cam_offset.x + vp.width * 0.5) / ts).ceil() as i32 + 1;
        let end_y = ((cam_offset.y + vp.height * 0.5) / ts).ceil() as i32 + 1;

        for ty in start_y..end_y {
            for tx in start_x..end_x {
                // Resolve wrapping.
                let mut wtx = tx;
                let mut wty = ty;
                if self.wrap_x {
                    wtx = wtx.rem_euclid(self.layer.width as i32);
                }
                if self.wrap_y {
                    wty = wty.rem_euclid(self.layer.height as i32);
                }

                if wtx < 0
                    || wty < 0
                    || wtx >= self.layer.width as i32
                    || wty >= self.layer.height as i32
                {
                    continue;
                }

                if let Some(_tile_id) = self.layer.get_tile(wtx as u32, wty as u32) {
                    // In a full implementation, we would look up the tile sprite from a tileset
                    // and draw it. For now, draw a colored rectangle as a placeholder.
                    let world_x = tx as f32 * ts - cam_offset.x + vp.width * 0.5;
                    let world_y = ty as f32 * ts - cam_offset.y + vp.height * 0.5;

                    renderer.draw_rect_filled(
                        Rect::new(world_x, world_y, ts, ts),
                        Color::rgb(100, 80, 60),
                    );
                }
            }
        }
    }

    fn z_order(&self) -> i32 {
        self.z
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn position(&self) -> Vec2 {
        self.offset
    }

    fn node_type(&self) -> SceneNodeType {
        SceneNodeType::TilePlane
    }
}
