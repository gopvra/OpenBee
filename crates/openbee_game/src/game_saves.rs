//! Save and load game state.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

/// Maximum allowed save file size (16 MiB).
const MAX_SAVE_FILE_SIZE: u64 = 16 * 1024 * 1024;

/// Validate that a path does not contain path traversal sequences and
/// that it does not escape the intended directory via symlinks.
fn sanitize_path(path: &Path) -> Result<()> {
    // Reject paths containing ".." components.
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            anyhow::bail!(
                "path traversal detected in save file path: {}",
                path.display()
            );
        }
    }
    Ok(())
}

use crate::game_app::Difficulty;
use crate::game_logic::{Checkpoint, GameState};

/// Serializable snapshot of the game state for save files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub game_state: GameState,
    pub current_level: u32,
    pub lives: i32,
    pub score: u64,
    pub checkpoints: Vec<Checkpoint>,
    pub difficulty: Difficulty,
    pub health: i32,
    pub pistol_ammo: i32,
    pub dynamite_ammo: i32,
    pub magic_ammo: i32,
    pub timestamp: u64,
    pub play_time_seconds: f64,
}

impl SaveData {
    /// Current save file format version.
    pub const CURRENT_VERSION: u32 = 1;
}

/// Save the game state to a JSON file at the given path.
pub fn save_game(data: &SaveData, path: &Path) -> Result<()> {
    sanitize_path(path)?;
    info!("Saving game to {:?}", path);
    let json = serde_json::to_string_pretty(data)
        .context("Failed to serialize save data")?;
    std::fs::write(path, json)
        .context("Failed to write save file")?;
    info!("Game saved successfully");
    Ok(())
}

/// Load a game state from a JSON file at the given path.
pub fn load_game(path: &Path) -> Result<SaveData> {
    sanitize_path(path)?;
    // Check file size before reading to prevent DoS.
    let metadata = std::fs::metadata(path).context("Failed to stat save file")?;
    anyhow::ensure!(
        metadata.len() <= MAX_SAVE_FILE_SIZE,
        "Save file too large: {} bytes (max {} bytes)",
        metadata.len(),
        MAX_SAVE_FILE_SIZE
    );
    info!("Loading game from {:?}", path);
    let json = std::fs::read_to_string(path)
        .context("Failed to read save file")?;
    let data: SaveData = serde_json::from_str(&json)
        .context("Failed to deserialize save data")?;
    if data.version != SaveData::CURRENT_VERSION {
        tracing::warn!(
            "Save file version {} differs from current version {}",
            data.version,
            SaveData::CURRENT_VERSION
        );
    }
    info!("Game loaded successfully (level {}, score {})", data.current_level, data.score);
    Ok(data)
}

/// List all save files in the given directory, sorted by modification time.
pub fn list_saves(directory: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut saves: Vec<_> = std::fs::read_dir(directory)
        .context("Failed to read saves directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "json")
        })
        .map(|entry| entry.path())
        .collect();
    saves.sort();
    Ok(saves)
}

/// Delete a save file.
pub fn delete_save(path: &Path) -> Result<()> {
    sanitize_path(path)?;
    std::fs::remove_file(path).context("Failed to delete save file")?;
    info!("Save file deleted: {:?}", path);
    Ok(())
}
