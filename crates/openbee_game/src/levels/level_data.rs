//! Level data structures parsed from WWD (WAP World Description) files.

use glam::Vec2;

/// A placed actor instance within a level.
#[derive(Debug, Clone)]
pub struct LevelActor {
    /// Template/logic name of the actor.
    pub logic: String,
    /// Image set for the actor.
    pub image_set: String,
    /// Animation set for the actor.
    pub animation_set: String,
    /// Position in the level.
    pub position: Vec2,
    /// Z-order for rendering.
    pub z_order: i32,
    /// Custom properties from the level editor.
    pub properties: Vec<(String, String)>,
}

/// A parallax plane in the level (background, action, foreground).
#[derive(Debug, Clone)]
pub struct LevelPlane {
    /// Name of the plane.
    pub name: String,
    /// Width in tiles.
    pub width_tiles: u32,
    /// Height in tiles.
    pub height_tiles: u32,
    /// Tile width in pixels.
    pub tile_width: u32,
    /// Tile height in pixels.
    pub tile_height: u32,
    /// Parallax scroll speed (1.0 = same as camera).
    pub scroll_speed_x: f32,
    /// Vertical parallax scroll speed.
    pub scroll_speed_y: f32,
    /// Z-order for rendering order.
    pub z_order: i32,
    /// Tile IDs in row-major order.
    pub tiles: Vec<i32>,
}

/// Full level data parsed from a WWD file.
#[derive(Debug, Clone)]
pub struct LevelData {
    /// Display name of the level.
    pub name: String,
    /// Total level width in pixels.
    pub width: u32,
    /// Total level height in pixels.
    pub height: u32,
    /// Width of each tile in pixels.
    pub tile_width: u32,
    /// Height of each tile in pixels.
    pub tile_height: u32,
    /// Player starting position.
    pub player_start: Vec2,
    /// Path to the level music.
    pub music_path: String,
    /// All placed actors in the level.
    pub actors: Vec<LevelActor>,
    /// Parallax planes (background, action, foreground layers).
    pub planes: Vec<LevelPlane>,
}

impl LevelData {
    /// Get the action plane (the main gameplay plane, typically index 1).
    pub fn action_plane(&self) -> Option<&LevelPlane> {
        self.planes.iter().find(|p| p.name == "Action" || p.z_order == 0)
    }

    /// Get the level bounds as a rectangle.
    pub fn bounds(&self) -> openbee_core::render::Rect {
        openbee_core::render::Rect::new(0.0, 0.0, self.width as f32, self.height as f32)
    }

    /// Number of actors in the level.
    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }
}
