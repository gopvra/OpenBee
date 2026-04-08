//! Mod discovery and loading — scans the mods directory, resolves load order,
//! and activates / deactivates individual mods.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};

use crate::mod_manifest::ModManifest;

/// A mod that has been discovered and optionally loaded.
#[derive(Debug, Clone)]
pub struct LoadedMod {
    /// Parsed manifest.
    pub manifest: ModManifest,
    /// Filesystem path to the mod directory.
    pub path: PathBuf,
    /// Whether the mod is currently enabled.
    pub enabled: bool,
    /// Position in the load order (lower = earlier).
    pub load_order: u32,
}

/// Discovers and manages mods on disk.
pub struct ModLoader {
    /// Root directory containing mod sub-directories.
    pub mods_directory: PathBuf,
    /// All discovered and/or loaded mods.
    pub loaded_mods: Vec<LoadedMod>,
}

impl ModLoader {
    /// Create a new loader pointing at the given mods directory.
    pub fn new(mods_dir: PathBuf) -> Self {
        Self {
            mods_directory: mods_dir,
            loaded_mods: Vec::new(),
        }
    }

    /// Scan the mods directory for sub-directories containing a `mod.json` manifest.
    /// Returns the list of discovered manifests (does **not** enable them).
    pub fn discover_mods(&mut self) -> Result<Vec<ModManifest>> {
        info!("Scanning for mods in {:?}", self.mods_directory);
        let mut manifests = Vec::new();

        if !self.mods_directory.exists() {
            warn!("Mods directory does not exist: {:?}", self.mods_directory);
            return Ok(manifests);
        }

        let entries = fs::read_dir(&self.mods_directory)
            .with_context(|| format!("Failed to read mods directory {:?}", self.mods_directory))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("mod.json");
            if !manifest_path.exists() {
                debug!("Skipping {:?} — no mod.json", path);
                continue;
            }

            match fs::read_to_string(&manifest_path) {
                Ok(json) => match ModManifest::from_json(&json) {
                    Ok(manifest) => {
                        info!(
                            "Discovered mod '{}' v{} at {:?}",
                            manifest.name, manifest.version, path
                        );

                        // Only add if not already known.
                        if !self
                            .loaded_mods
                            .iter()
                            .any(|m| m.manifest.id == manifest.id)
                        {
                            let order = self.loaded_mods.len() as u32;
                            self.loaded_mods.push(LoadedMod {
                                manifest: manifest.clone(),
                                path: path.clone(),
                                enabled: false,
                                load_order: order,
                            });
                        }
                        manifests.push(manifest);
                    }
                    Err(e) => {
                        error!("Failed to parse {:?}: {e}", manifest_path);
                    }
                },
                Err(e) => {
                    error!("Failed to read {:?}: {e}", manifest_path);
                }
            }
        }

        info!("Discovered {} mod(s)", manifests.len());
        Ok(manifests)
    }

    /// Enable (load) a mod by its ID. The mod must have been previously discovered.
    pub fn load_mod(&mut self, mod_id: &str) -> Result<()> {
        // First pass: read-only check for existence and enabled state, and
        // extract the data we need for conflict detection.
        let entry = self
            .loaded_mods
            .iter()
            .find(|m| m.manifest.id == mod_id)
            .ok_or_else(|| anyhow::anyhow!("Mod '{mod_id}' not found"))?;

        if entry.enabled {
            info!("Mod '{}' is already loaded", entry.manifest.name);
            return Ok(());
        }

        let entry_name = entry.manifest.name.clone();
        let entry_conflicts = entry.manifest.conflicts.clone();
        let entry_id = entry.manifest.id.clone();

        // Check for conflicts with already-enabled mods.
        let conflicts: Vec<String> = self
            .loaded_mods
            .iter()
            .filter(|m| m.enabled && m.manifest.id != mod_id)
            .filter(|m| {
                entry_conflicts.contains(&m.manifest.id) || m.manifest.conflicts.contains(&entry_id)
            })
            .map(|m| m.manifest.name.clone())
            .collect();

        if !conflicts.is_empty() {
            return Err(anyhow::anyhow!(
                "Mod '{}' conflicts with: {}",
                entry_name,
                conflicts.join(", ")
            ));
        }

        // Now mutably borrow to set enabled.
        let entry = self
            .loaded_mods
            .iter_mut()
            .find(|m| m.manifest.id == mod_id)
            .ok_or_else(|| anyhow::anyhow!("Mod '{mod_id}' not found"))?;
        entry.enabled = true;
        info!("Loaded mod '{}'", entry.manifest.name);
        Ok(())
    }

    /// Disable (unload) a mod by its ID.
    pub fn unload_mod(&mut self, mod_id: &str) -> Result<()> {
        let entry = self
            .loaded_mods
            .iter_mut()
            .find(|m| m.manifest.id == mod_id)
            .ok_or_else(|| anyhow::anyhow!("Mod '{mod_id}' not found"))?;

        entry.enabled = false;
        info!("Unloaded mod '{}'", entry.manifest.name);
        Ok(())
    }

    /// Look up a loaded mod by ID.
    pub fn get_mod(&self, mod_id: &str) -> Option<&LoadedMod> {
        self.loaded_mods.iter().find(|m| m.manifest.id == mod_id)
    }

    /// Return a slice of all known mods.
    pub fn list_mods(&self) -> &[LoadedMod] {
        &self.loaded_mods
    }

    /// Sort `loaded_mods` so that dependencies come before dependents.
    /// Uses a simple topological-sort approach.
    pub fn resolve_load_order(&mut self) -> Result<()> {
        let n = self.loaded_mods.len();
        let mut ordered: Vec<usize> = Vec::with_capacity(n);
        let mut visited = vec![false; n];

        // Build an index map: mod_id -> index.
        let id_to_idx: std::collections::HashMap<String, usize> = self
            .loaded_mods
            .iter()
            .enumerate()
            .map(|(i, m)| (m.manifest.id.clone(), i))
            .collect();

        fn visit(
            idx: usize,
            mods: &[LoadedMod],
            id_to_idx: &std::collections::HashMap<String, usize>,
            visited: &mut [bool],
            ordered: &mut Vec<usize>,
            stack: &mut Vec<usize>,
        ) -> Result<()> {
            if visited[idx] {
                return Ok(());
            }
            if stack.contains(&idx) {
                return Err(anyhow::anyhow!(
                    "Circular dependency detected involving '{}'",
                    mods[idx].manifest.id
                ));
            }
            stack.push(idx);

            for dep in &mods[idx].manifest.dependencies {
                if let Some(&dep_idx) = id_to_idx.get(&dep.mod_id) {
                    visit(dep_idx, mods, id_to_idx, visited, ordered, stack)?;
                }
                // Missing optional deps are silently ignored.
            }

            stack.pop();
            visited[idx] = true;
            ordered.push(idx);
            Ok(())
        }

        for i in 0..n {
            if !visited[i] {
                let mut stack = Vec::new();
                visit(
                    i,
                    &self.loaded_mods,
                    &id_to_idx,
                    &mut visited,
                    &mut ordered,
                    &mut stack,
                )?;
            }
        }

        // Assign load_order based on the resolved ordering.
        for (order, &idx) in ordered.iter().enumerate() {
            self.loaded_mods[idx].load_order = order as u32;
        }

        // Sort the vec in-place by load_order.
        self.loaded_mods.sort_by_key(|m| m.load_order);

        info!("Load order resolved for {} mods", n);
        Ok(())
    }
}
