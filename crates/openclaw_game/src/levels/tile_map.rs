//! Tile map for level geometry and collision.

use serde::{Deserialize, Serialize};

/// Tile collision type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileCollision {
    /// No collision (empty space).
    None,
    /// Full solid tile.
    Solid,
    /// One-way platform (can jump through from below).
    OneWayPlatform,
    /// Climbable surface (ladder/vine).
    Climbable,
    /// Death zone (spikes, lava, bottomless pit).
    Death,
    /// Water tile (swimming physics).
    Water,
}

/// A single tile in the map.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    /// Sprite/texture ID for rendering (-1 = no tile).
    pub tile_id: i32,
    /// Collision type for physics.
    pub collision: TileCollision,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            tile_id: -1,
            collision: TileCollision::None,
        }
    }
}

/// 2D grid of tiles representing the level geometry.
pub struct TileMap {
    /// Width in tiles.
    width: usize,
    /// Height in tiles.
    height: usize,
    /// Width of each tile in pixels.
    tile_width: u32,
    /// Height of each tile in pixels.
    tile_height: u32,
    /// Flat array of tiles in row-major order.
    tiles: Vec<Tile>,
}

impl TileMap {
    /// Create a new empty tile map.
    pub fn new(width: usize, height: usize, tile_width: u32, tile_height: u32) -> Self {
        Self {
            width,
            height,
            tile_width,
            tile_height,
            tiles: vec![Tile::default(); width * height],
        }
    }

    /// Get a tile at grid coordinates.
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.width && y < self.height {
            Some(&self.tiles[y * self.width + x])
        } else {
            None
        }
    }

    /// Set a tile at grid coordinates.
    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    /// Get the tile at a world-space pixel position.
    pub fn get_tile_at_world(&self, world_x: f32, world_y: f32) -> Option<&Tile> {
        if world_x < 0.0 || world_y < 0.0 {
            return None;
        }
        let tx = (world_x / self.tile_width as f32) as usize;
        let ty = (world_y / self.tile_height as f32) as usize;
        self.get_tile(tx, ty)
    }

    /// Query collision type at a world-space pixel position.
    pub fn collision_at(&self, world_x: f32, world_y: f32) -> TileCollision {
        self.get_tile_at_world(world_x, world_y)
            .map(|t| t.collision)
            .unwrap_or(TileCollision::None)
    }

    /// Check if a rectangle overlaps any solid tiles.
    pub fn rect_collides_solid(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        let start_tx = ((x / self.tile_width as f32).floor() as usize).max(0);
        let start_ty = ((y / self.tile_height as f32).floor() as usize).max(0);
        let end_tx = (((x + width) / self.tile_width as f32).ceil() as usize).min(self.width);
        let end_ty = (((y + height) / self.tile_height as f32).ceil() as usize).min(self.height);

        for ty in start_ty..end_ty {
            for tx in start_tx..end_tx {
                if let Some(tile) = self.get_tile(tx, ty) {
                    if tile.collision == TileCollision::Solid {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if there is solid ground directly below a position.
    pub fn is_ground_below(&self, world_x: f32, world_y: f32) -> bool {
        let check_y = world_y + 1.0;
        matches!(
            self.collision_at(world_x, check_y),
            TileCollision::Solid | TileCollision::OneWayPlatform
        )
    }

    /// Grid width in tiles.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Grid height in tiles.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Tile width in pixels.
    pub fn tile_width(&self) -> u32 {
        self.tile_width
    }

    /// Tile height in pixels.
    pub fn tile_height(&self) -> u32 {
        self.tile_height
    }

    /// Total width in pixels.
    pub fn pixel_width(&self) -> u32 {
        self.width as u32 * self.tile_width
    }

    /// Total height in pixels.
    pub fn pixel_height(&self) -> u32 {
        self.height as u32 * self.tile_height
    }
}
