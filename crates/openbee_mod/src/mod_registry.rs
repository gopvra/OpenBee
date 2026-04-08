//! Mod registry — tracks active mods and provides queries.

use std::collections::HashMap;

use tracing::info;

use crate::mod_manifest::ModManifest;

/// An entry in the registry for a single active mod.
#[derive(Debug, Clone)]
pub struct ModRegistryEntry {
    pub manifest: ModManifest,
    pub enabled: bool,
    pub load_order: u32,
}

/// Central registry of all known mods and their activation state.
pub struct ModRegistry {
    entries: HashMap<String, ModRegistryEntry>,
}

impl ModRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a mod manifest. If the mod is already registered, its entry is
    /// updated.
    pub fn register(&mut self, manifest: ModManifest, load_order: u32) {
        let id = manifest.id.clone();
        info!("Registering mod '{}' (order={load_order})", manifest.name);
        self.entries.insert(
            id,
            ModRegistryEntry {
                manifest,
                enabled: false,
                load_order,
            },
        );
    }

    /// Enable a mod by ID.
    pub fn enable(&mut self, mod_id: &str) -> bool {
        if let Some(entry) = self.entries.get_mut(mod_id) {
            entry.enabled = true;
            info!("Enabled mod '{}'", entry.manifest.name);
            true
        } else {
            false
        }
    }

    /// Disable a mod by ID.
    pub fn disable(&mut self, mod_id: &str) -> bool {
        if let Some(entry) = self.entries.get_mut(mod_id) {
            entry.enabled = false;
            info!("Disabled mod '{}'", entry.manifest.name);
            true
        } else {
            false
        }
    }

    /// Check whether a mod is currently enabled.
    pub fn is_enabled(&self, mod_id: &str) -> bool {
        self.entries
            .get(mod_id)
            .is_some_and(|e| e.enabled)
    }

    /// Get the registry entry for a mod.
    pub fn get(&self, mod_id: &str) -> Option<&ModRegistryEntry> {
        self.entries.get(mod_id)
    }

    /// Return all registered mod IDs.
    pub fn mod_ids(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    /// Return all enabled mods sorted by load order.
    pub fn enabled_mods(&self) -> Vec<&ModRegistryEntry> {
        let mut mods: Vec<&ModRegistryEntry> =
            self.entries.values().filter(|e| e.enabled).collect();
        mods.sort_by_key(|e| e.load_order);
        mods
    }

    /// Return all registered mods sorted by load order.
    pub fn all_mods(&self) -> Vec<&ModRegistryEntry> {
        let mut mods: Vec<&ModRegistryEntry> = self.entries.values().collect();
        mods.sort_by_key(|e| e.load_order);
        mods
    }

    /// Total number of registered mods.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Number of currently enabled mods.
    pub fn enabled_count(&self) -> usize {
        self.entries.values().filter(|e| e.enabled).count()
    }

    /// Remove a mod from the registry entirely.
    pub fn unregister(&mut self, mod_id: &str) -> bool {
        self.entries.remove(mod_id).is_some()
    }
}

impl Default for ModRegistry {
    fn default() -> Self {
        Self::new()
    }
}
