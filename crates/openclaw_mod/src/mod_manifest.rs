//! Mod metadata manifest — parsed from a `mod.json` file in each mod directory.

use serde::{Deserialize, Serialize};

/// How a modded asset interacts with the original.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverrideMode {
    /// Completely replace the original asset.
    Replace,
    /// Append data to the original (e.g. add entries to a list).
    Append,
    /// Deep-merge with the original (e.g. JSON patch).
    Merge,
}

impl Default for OverrideMode {
    fn default() -> Self {
        Self::Replace
    }
}

/// A dependency on another mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    /// ID of the required mod.
    pub mod_id: String,
    /// Minimum version of the dependency (semver-ish string), if any.
    pub min_version: Option<String>,
}

/// An asset mapping entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    /// Path of the asset inside the mod directory.
    pub source: String,
    /// Engine asset path that this overrides / extends.
    pub target: String,
    /// How the override should be applied.
    #[serde(default)]
    pub override_mode: OverrideMode,
}

/// Complete mod metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    /// Unique mod identifier (e.g. `"com.example.mymod"`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Semver version string.
    pub version: String,
    /// Author name or handle.
    pub author: String,
    /// Short description.
    pub description: String,
    /// Mods that must be loaded before this one.
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    /// Mod IDs that are incompatible with this mod.
    #[serde(default)]
    pub conflicts: Vec<String>,
    /// Asset override / addition entries.
    #[serde(default)]
    pub assets: Vec<AssetEntry>,
    /// Lua script files to load (paths relative to the mod directory).
    #[serde(default)]
    pub scripts: Vec<String>,
    /// Minimum engine version required for this mod.
    #[serde(default)]
    pub min_engine_version: Option<String>,
}

impl ModManifest {
    /// Load a manifest from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialise the manifest to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
