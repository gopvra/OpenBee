//! Asset override / overlay system — lets mods replace or extend engine assets
//! without modifying the originals.

use std::collections::HashMap;

use tracing::{debug, info};

use crate::mod_manifest::OverrideMode;

/// Describes a single asset override registered by a mod.
#[derive(Debug, Clone)]
pub struct AssetOverride {
    /// Original engine asset path that is being overridden.
    pub original_path: String,
    /// ID of the mod that owns this override.
    pub mod_id: String,
    /// Path within the mod directory that supplies the replacement data.
    pub mod_path: String,
    /// How the override is applied.
    pub override_mode: OverrideMode,
}

/// Manages all active asset overrides across loaded mods.
pub struct AssetOverrideManager {
    /// Map from original engine asset path to its override descriptor.
    /// If multiple mods override the same asset, the last one registered wins
    /// (respecting load order).
    pub overrides: HashMap<String, AssetOverride>,
}

impl AssetOverrideManager {
    /// Create an empty override manager.
    pub fn new() -> Self {
        Self {
            overrides: HashMap::new(),
        }
    }

    /// Register an asset override. If the same `original` path already has an
    /// override from a different mod, the new one replaces it (later load order
    /// wins).
    pub fn register_override(
        &mut self,
        original: &str,
        mod_id: &str,
        mod_path: &str,
        mode: OverrideMode,
    ) {
        debug!(
            "Registering asset override: '{original}' -> '{mod_path}' (mod={mod_id}, mode={mode:?})"
        );
        self.overrides.insert(
            original.to_string(),
            AssetOverride {
                original_path: original.to_string(),
                mod_id: mod_id.to_string(),
                mod_path: mod_path.to_string(),
                override_mode: mode,
            },
        );
    }

    /// Resolve an asset path — returns the mod-supplied path if an override
    /// exists, otherwise returns the original path.
    pub fn resolve_path<'a>(&'a self, original_path: &'a str) -> &'a str {
        self.overrides
            .get(original_path)
            .map(|o| o.mod_path.as_str())
            .unwrap_or(original_path)
    }

    /// Check whether a given path has an active override.
    pub fn has_override(&self, path: &str) -> bool {
        self.overrides.contains_key(path)
    }

    /// Remove all overrides that belong to a specific mod.
    pub fn clear_mod_overrides(&mut self, mod_id: &str) {
        let before = self.overrides.len();
        self.overrides.retain(|_, v| v.mod_id != mod_id);
        let removed = before - self.overrides.len();
        if removed > 0 {
            info!("Cleared {removed} asset override(s) for mod '{mod_id}'");
        }
    }

    /// Remove all overrides.
    pub fn clear_all(&mut self) {
        self.overrides.clear();
    }

    /// Number of active overrides.
    pub fn count(&self) -> usize {
        self.overrides.len()
    }

    /// Iterate over all active overrides.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &AssetOverride)> {
        self.overrides.iter().map(|(k, v)| (k.as_str(), v))
    }
}

impl Default for AssetOverrideManager {
    fn default() -> Self {
        Self::new()
    }
}
