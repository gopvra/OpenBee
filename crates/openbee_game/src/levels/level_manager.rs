//! Level manager: loading, transitioning, and querying level state.

use anyhow::{Context, Result};
use glam::Vec2;
use tracing::info;

use super::level_data::LevelData;
use super::tile_map::TileMap;

/// Level metadata for each of the 14 Captain Claw levels.
#[derive(Debug, Clone)]
pub struct LevelInfo {
    /// Level number (1-14).
    pub number: u32,
    /// Display name of the level.
    pub name: String,
    /// Resource path prefix for level assets.
    pub resource_prefix: String,
    /// Music track filename.
    pub music: String,
    /// Boss type, if any.
    pub has_boss: bool,
}

/// Manages level loading, transitions, and provides access to current level data.
pub struct LevelManager {
    /// All level definitions.
    level_infos: Vec<LevelInfo>,
    /// Currently loaded level data.
    current_level: Option<LevelData>,
    /// Currently loaded tile map.
    current_tile_map: Option<TileMap>,
    /// Level number of the currently loaded level.
    current_level_number: u32,
}

impl LevelManager {
    /// Create a new level manager with all 14 level definitions.
    pub fn new() -> Self {
        let level_infos = vec![
            LevelInfo {
                number: 1,
                name: "La Roca".to_string(),
                resource_prefix: "LEVEL1".to_string(),
                music: "LEVEL1_MUSIC".to_string(),
                has_boss: false,
            },
            LevelInfo {
                number: 2,
                name: "The Battlements".to_string(),
                resource_prefix: "LEVEL2".to_string(),
                music: "LEVEL2_MUSIC".to_string(),
                has_boss: true,
            },
            LevelInfo {
                number: 3,
                name: "The Footpath".to_string(),
                resource_prefix: "LEVEL3".to_string(),
                music: "LEVEL3_MUSIC".to_string(),
                has_boss: false,
            },
            LevelInfo {
                number: 4,
                name: "The Dark Woods".to_string(),
                resource_prefix: "LEVEL4".to_string(),
                music: "LEVEL4_MUSIC".to_string(),
                has_boss: true,
            },
            LevelInfo {
                number: 5,
                name: "The Township".to_string(),
                resource_prefix: "LEVEL5".to_string(),
                music: "LEVEL5_MUSIC".to_string(),
                has_boss: false,
            },
            LevelInfo {
                number: 6,
                name: "El Puerto del Lobo".to_string(),
                resource_prefix: "LEVEL6".to_string(),
                music: "LEVEL6_MUSIC".to_string(),
                has_boss: true,
            },
            LevelInfo {
                number: 7,
                name: "The Docks".to_string(),
                resource_prefix: "LEVEL7".to_string(),
                music: "LEVEL7_MUSIC".to_string(),
                has_boss: false,
            },
            LevelInfo {
                number: 8,
                name: "The Shipyard".to_string(),
                resource_prefix: "LEVEL8".to_string(),
                music: "LEVEL8_MUSIC".to_string(),
                has_boss: false,
            },
            LevelInfo {
                number: 9,
                name: "The Pirate's Cove".to_string(),
                resource_prefix: "LEVEL9".to_string(),
                music: "LEVEL9_MUSIC".to_string(),
                has_boss: true,
            },
            LevelInfo {
                number: 10,
                name: "The Cliffs".to_string(),
                resource_prefix: "LEVEL10".to_string(),
                music: "LEVEL10_MUSIC".to_string(),
                has_boss: false,
            },
            LevelInfo {
                number: 11,
                name: "The Caverns".to_string(),
                resource_prefix: "LEVEL11".to_string(),
                music: "LEVEL11_MUSIC".to_string(),
                has_boss: true,
            },
            LevelInfo {
                number: 12,
                name: "The Undersea Caves".to_string(),
                resource_prefix: "LEVEL12".to_string(),
                music: "LEVEL12_MUSIC".to_string(),
                has_boss: true,
            },
            LevelInfo {
                number: 13,
                name: "Tiger Island".to_string(),
                resource_prefix: "LEVEL13".to_string(),
                music: "LEVEL13_MUSIC".to_string(),
                has_boss: true,
            },
            LevelInfo {
                number: 14,
                name: "Omar's Lair".to_string(),
                resource_prefix: "LEVEL14".to_string(),
                music: "LEVEL14_MUSIC".to_string(),
                has_boss: true,
            },
        ];

        Self {
            level_infos,
            current_level: None,
            current_tile_map: None,
            current_level_number: 0,
        }
    }

    /// Load a level by number. Parses WWD data and constructs the tile map.
    pub fn load_level(&mut self, level_number: u32) -> Result<()> {
        let info = self
            .level_infos
            .iter()
            .find(|l| l.number == level_number)
            .with_context(|| format!("Invalid level number: {}", level_number))?
            .clone();

        info!("Loading level {}: {}", info.number, info.name);

        // In a full implementation this would load from the REZ archive via openbee_rez.
        let level_data = LevelData {
            name: info.name.clone(),
            width: 20000,
            height: 5000,
            tile_width: 64,
            tile_height: 64,
            player_start: Vec2::new(200.0, 4000.0),
            music_path: info.music.clone(),
            actors: Vec::new(),
            planes: Vec::new(),
        };

        let tiles_x = (level_data.width / level_data.tile_width) as usize;
        let tiles_y = (level_data.height / level_data.tile_height) as usize;
        let tile_map = TileMap::new(
            tiles_x,
            tiles_y,
            level_data.tile_width,
            level_data.tile_height,
        );

        self.current_level = Some(level_data);
        self.current_tile_map = Some(tile_map);
        self.current_level_number = level_number;

        info!("Level {} loaded successfully", level_number);
        Ok(())
    }

    /// Transition to a new level, unloading the current one.
    pub fn transition_to(&mut self, level_number: u32) -> Result<()> {
        self.unload_current();
        self.load_level(level_number)
    }

    /// Unload the current level data.
    pub fn unload_current(&mut self) {
        self.current_level = None;
        self.current_tile_map = None;
        self.current_level_number = 0;
    }

    /// Get a reference to the currently loaded level data.
    pub fn current_level_data(&self) -> Option<&LevelData> {
        self.current_level.as_ref()
    }

    /// Get a reference to the current tile map.
    pub fn current_tile_map(&self) -> Option<&TileMap> {
        self.current_tile_map.as_ref()
    }

    /// Get a mutable reference to the current tile map.
    pub fn current_tile_map_mut(&mut self) -> Option<&mut TileMap> {
        self.current_tile_map.as_mut()
    }

    /// Get info about a specific level.
    pub fn level_info(&self, level_number: u32) -> Option<&LevelInfo> {
        self.level_infos.iter().find(|l| l.number == level_number)
    }

    /// Current level number (0 = none loaded).
    pub fn current_level_number(&self) -> u32 {
        self.current_level_number
    }

    /// Total number of levels.
    pub fn total_levels(&self) -> u32 {
        self.level_infos.len() as u32
    }
}

impl Default for LevelManager {
    fn default() -> Self {
        Self::new()
    }
}
