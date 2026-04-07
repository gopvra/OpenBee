//! Hot reload system for live-reloading assets and scripts without restarting.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::info;

/// Tracks file modification times and detects changes for hot reload.
pub struct HotReloadWatcher {
    pub watched_files: HashMap<PathBuf, SystemTime>,
    pub changed_files: Vec<PathBuf>,
    pub watch_directories: Vec<PathBuf>,
    pub poll_interval: f32,
    pub poll_timer: f32,
    pub enabled: bool,
    /// Only watch files with these extensions (e.g. "lua", "xml", "png").
    pub file_extensions: HashSet<String>,
}

impl HotReloadWatcher {
    /// Create a new watcher that polls at the given interval (in seconds).
    pub fn new(poll_interval: f32) -> Self {
        Self {
            watched_files: HashMap::new(),
            changed_files: Vec::new(),
            watch_directories: Vec::new(),
            poll_interval,
            poll_timer: 0.0,
            enabled: true,
            file_extensions: HashSet::new(),
        }
    }

    /// Register an entire directory tree for watching.
    pub fn watch_directory(&mut self, path: &Path) {
        if !self.watch_directories.contains(&path.to_path_buf()) {
            self.watch_directories.push(path.to_path_buf());
            self.scan_directory(path);
        }
    }

    /// Register a single file for watching.
    pub fn watch_file(&mut self, path: &Path) {
        if let Ok(meta) = std::fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                self.watched_files.insert(path.to_path_buf(), modified);
            }
        }
    }

    /// Add a file extension to the watch filter (without the leading dot).
    pub fn add_extension(&mut self, ext: &str) {
        self.file_extensions.insert(ext.to_lowercase());
    }

    /// Advance the poll timer by `dt` seconds and, if the interval has elapsed,
    /// check for changes. Returns the list of changed file paths since the last poll.
    pub fn update(&mut self, dt: f32) -> &[PathBuf] {
        self.changed_files.clear();

        if !self.enabled {
            return &self.changed_files;
        }

        self.poll_timer += dt;
        if self.poll_timer >= self.poll_interval {
            self.poll_timer = 0.0;
            self.check_for_changes();
        }

        &self.changed_files
    }

    /// Whether any files changed in the most recent poll.
    pub fn has_changes(&self) -> bool {
        !self.changed_files.is_empty()
    }

    /// Clear the list of changed files.
    pub fn clear_changes(&mut self) {
        self.changed_files.clear();
    }

    /// Enable the watcher.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the watcher; no polling will occur while disabled.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check every watched file for modification time changes.
    fn check_for_changes(&mut self) {
        // Re-scan watched directories in case new files appeared.
        let dirs: Vec<PathBuf> = self.watch_directories.clone();
        for dir in &dirs {
            self.scan_directory(dir);
        }

        let paths: Vec<PathBuf> = self.watched_files.keys().cloned().collect();
        for path in paths {
            if let Ok(meta) = std::fs::metadata(&path) {
                if let Ok(modified) = meta.modified() {
                    if let Some(prev) = self.watched_files.get(&path) {
                        if modified > *prev {
                            self.changed_files.push(path.clone());
                            self.watched_files.insert(path, modified);
                        }
                    }
                }
            }
        }
    }

    /// Recursively scan a directory and register any files whose extension matches the filter.
    fn scan_directory(&mut self, dir: &Path) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_directory(&path);
            } else if path.is_file() {
                // If extension filter is non-empty, only include matching files.
                if !self.file_extensions.is_empty() {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if !self.file_extensions.contains(&ext) {
                        continue;
                    }
                }

                if !self.watched_files.contains_key(&path) {
                    if let Ok(meta) = std::fs::metadata(&path) {
                        if let Ok(modified) = meta.modified() {
                            self.watched_files.insert(path, modified);
                        }
                    }
                }
            }
        }
    }
}

/// Categories of reloadable resources, determined by file extension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReloadableType {
    LuaScript,
    XmlTemplate,
    Palette,
    Animation,
    Sound,
    Music,
    Level,
    Config,
}

impl ReloadableType {
    /// Determine the reloadable type from a file extension (without the leading dot).
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "lua" => Some(Self::LuaScript),
            "xml" => Some(Self::XmlTemplate),
            "pal" | "palette" => Some(Self::Palette),
            "ani" | "anim" => Some(Self::Animation),
            "wav" | "ogg" => Some(Self::Sound),
            "mid" | "midi" | "mp3" => Some(Self::Music),
            "lvl" | "level" | "wwd" => Some(Self::Level),
            "json" | "toml" | "cfg" | "ini" => Some(Self::Config),
            _ => None,
        }
    }

    /// Return the file extensions associated with this reloadable type.
    pub fn extensions(&self) -> &[&str] {
        match self {
            Self::LuaScript => &["lua"],
            Self::XmlTemplate => &["xml"],
            Self::Palette => &["pal", "palette"],
            Self::Animation => &["ani", "anim"],
            Self::Sound => &["wav", "ogg"],
            Self::Music => &["mid", "midi", "mp3"],
            Self::Level => &["lvl", "level", "wwd"],
            Self::Config => &["json", "toml", "cfg", "ini"],
        }
    }
}

/// Hot reload manager that owns a watcher and dispatches reload events.
pub struct HotReloadManager {
    pub watcher: HotReloadWatcher,
    pub reload_callbacks: HashMap<String, Vec<Box<dyn Fn(&Path) + Send>>>,
    pub reload_count: u64,
    pub last_reload_path: Option<PathBuf>,
}

impl HotReloadManager {
    /// Create a new manager with the watcher initially disabled.
    pub fn new() -> Self {
        Self {
            watcher: HotReloadWatcher::new(1.0),
            reload_callbacks: HashMap::new(),
            reload_count: 0,
            last_reload_path: None,
        }
    }

    /// Enable hot reloading with the given poll interval (in seconds).
    pub fn enable(&mut self, poll_interval: f32) {
        self.watcher.poll_interval = poll_interval;
        self.watcher.enable();
    }

    /// Disable hot reloading entirely.
    pub fn disable(&mut self) {
        self.watcher.disable();
    }

    /// Watch a path (file or directory).
    pub fn watch(&mut self, path: &Path) {
        if path.is_dir() {
            self.watcher.watch_directory(path);
        } else {
            self.watcher.watch_file(path);
        }
    }

    /// Advance the watcher by `dt` seconds and return a list of changed files
    /// along with their detected reloadable type. Also fires registered callbacks.
    pub fn update(&mut self, dt: f32) -> Vec<(PathBuf, ReloadableType)> {
        let changed: Vec<PathBuf> = self.watcher.update(dt).to_vec();
        let mut results = Vec::new();

        for path in &changed {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if let Some(reload_type) = ReloadableType::from_extension(&ext) {
                info!(path = %path.display(), ?reload_type, "Hot-reloading asset");
                results.push((path.clone(), reload_type));
                self.reload_count += 1;
                self.last_reload_path = Some(path.clone());

                // Fire callbacks registered for this extension.
                if let Some(callbacks) = self.reload_callbacks.get(&ext) {
                    for cb in callbacks {
                        cb(path);
                    }
                }
            }
        }

        results
    }

    /// Register a callback to be invoked whenever a file with the given extension is reloaded.
    pub fn on_reload(&mut self, extension: &str, callback: Box<dyn Fn(&Path) + Send>) {
        self.reload_callbacks
            .entry(extension.to_lowercase())
            .or_default()
            .push(callback);
    }
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reloadable_type_from_extension() {
        assert_eq!(
            ReloadableType::from_extension("lua"),
            Some(ReloadableType::LuaScript)
        );
        assert_eq!(
            ReloadableType::from_extension("XML"),
            Some(ReloadableType::XmlTemplate)
        );
        assert_eq!(
            ReloadableType::from_extension("json"),
            Some(ReloadableType::Config)
        );
        assert_eq!(ReloadableType::from_extension("exe"), None);
    }

    #[test]
    fn test_reloadable_type_extensions_roundtrip() {
        let types = [
            ReloadableType::LuaScript,
            ReloadableType::XmlTemplate,
            ReloadableType::Palette,
            ReloadableType::Animation,
            ReloadableType::Sound,
            ReloadableType::Music,
            ReloadableType::Level,
            ReloadableType::Config,
        ];
        for t in &types {
            for ext in t.extensions() {
                assert_eq!(ReloadableType::from_extension(ext), Some(*t));
            }
        }
    }

    #[test]
    fn test_watcher_disabled() {
        let mut watcher = HotReloadWatcher::new(0.1);
        watcher.disable();
        let changed = watcher.update(1.0);
        assert!(changed.is_empty());
    }

    #[test]
    fn test_add_extension() {
        let mut watcher = HotReloadWatcher::new(1.0);
        watcher.add_extension("lua");
        watcher.add_extension("XML");
        assert!(watcher.file_extensions.contains("lua"));
        assert!(watcher.file_extensions.contains("xml"));
    }

    #[test]
    fn test_manager_enable_disable() {
        let mut mgr = HotReloadManager::new();
        mgr.enable(0.5);
        assert!(mgr.watcher.enabled);
        assert_eq!(mgr.watcher.poll_interval, 0.5);
        mgr.disable();
        assert!(!mgr.watcher.enabled);
    }
}
