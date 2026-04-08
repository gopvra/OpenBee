//! Resource manager that coordinates loaders and caching.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};

use super::cache::ResourceCache;
use super::loaders::ResourceLoader;

/// Central resource manager: registers loaders by file extension, loads resources
/// through the appropriate loader, and caches results.
pub struct ResourceManager {
    /// Map of file extension (lowercase, no dot) to loader.
    loaders: HashMap<String, Box<dyn ResourceLoader>>,
    /// Resource cache.
    cache: ResourceCache,
}

impl ResourceManager {
    /// Create a new resource manager with the given cache memory budget (in bytes).
    pub fn new(cache_budget: usize) -> Self {
        Self {
            loaders: HashMap::new(),
            cache: ResourceCache::new(cache_budget),
        }
    }

    /// Register a resource loader. It will be used for all file extensions it reports.
    pub fn register_loader(&mut self, loader: Box<dyn ResourceLoader>) {
        for ext in loader.extensions() {
            self.loaders.insert(ext.to_lowercase(), loader.clone_box());
        }
    }

    /// Load a resource by path. Returns a cached result if available; otherwise uses
    /// the appropriate loader.
    ///
    /// The `data` parameter is the raw bytes of the file. The caller is responsible for
    /// reading the file from disk or from a REZ archive.
    pub fn load<T: Any + Send + Sync + 'static>(
        &mut self,
        path: &str,
        data: &[u8],
    ) -> Result<Arc<T>> {
        // Check cache first.
        if let Some(cached) = self.cache.get(path) {
            if let Ok(typed) = cached.downcast::<T>() {
                return Ok(typed);
            }
        }

        // Determine extension.
        let ext = path
            .rsplit('.')
            .next()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let loader = self
            .loaders
            .get(&ext)
            .ok_or_else(|| anyhow!("No loader registered for extension: .{}", ext))?;

        let loaded = loader.load(data, path)?;

        let typed = loaded
            .downcast::<T>()
            .map_err(|_| anyhow!("Loaded resource type mismatch for: {}", path))?;

        let arc: Arc<T> = Arc::from(typed);
        let size = std::mem::size_of::<T>(); // rough estimate
        self.cache.insert(
            path.to_string(),
            arc.clone() as Arc<dyn Any + Send + Sync>,
            size,
        );

        Ok(arc)
    }

    /// Check if a resource is already cached.
    pub fn has_resource(&self, path: &str) -> bool {
        self.cache.contains(path)
    }

    /// Remove a resource from the cache.
    pub fn unload(&mut self, path: &str) -> bool {
        self.cache.remove(path)
    }

    /// Get a mutable reference to the cache.
    pub fn cache_mut(&mut self) -> &mut ResourceCache {
        &mut self.cache
    }

    /// Get a shared reference to the cache.
    pub fn cache(&self) -> &ResourceCache {
        &self.cache
    }
}
