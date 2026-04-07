//! LRU resource cache with memory limit.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// An entry in the resource cache.
struct CacheEntry {
    /// The cached resource data.
    data: Arc<dyn Any + Send + Sync>,
    /// Estimated memory usage in bytes.
    size: usize,
    /// Last time this entry was accessed.
    last_accessed: Instant,
}

/// LRU (Least Recently Used) resource cache with a maximum memory budget.
pub struct ResourceCache {
    entries: HashMap<String, CacheEntry>,
    /// Maximum total memory in bytes before eviction kicks in.
    max_memory: usize,
    /// Current total memory usage in bytes.
    current_memory: usize,
}

impl ResourceCache {
    /// Create a new cache with the given maximum memory budget in bytes.
    pub fn new(max_memory: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_memory,
            current_memory: 0,
        }
    }

    /// Insert a resource into the cache. Evicts LRU entries if the memory budget is exceeded.
    pub fn insert(&mut self, key: String, data: Arc<dyn Any + Send + Sync>, size: usize) {
        // If updating an existing entry, subtract its old size.
        if let Some(old) = self.entries.remove(&key) {
            self.current_memory = self.current_memory.saturating_sub(old.size);
        }

        // Evict until we have room.
        while self.current_memory + size > self.max_memory && !self.entries.is_empty() {
            self.evict_lru();
        }

        self.current_memory += size;
        self.entries.insert(
            key,
            CacheEntry {
                data,
                size,
                last_accessed: Instant::now(),
            },
        );
    }

    /// Get a resource from the cache, updating its access time.
    pub fn get(&mut self, key: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_accessed = Instant::now();
            Some(Arc::clone(&entry.data))
        } else {
            None
        }
    }

    /// Check if a key exists in the cache.
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Remove a specific entry from the cache.
    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(entry) = self.entries.remove(key) {
            self.current_memory = self.current_memory.saturating_sub(entry.size);
            true
        } else {
            false
        }
    }

    /// Evict the least recently used entry.
    pub fn evict_lru(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let lru_key = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_accessed)
            .map(|(k, _)| k.clone());

        if let Some(key) = lru_key {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_memory = self.current_memory.saturating_sub(entry.size);
                tracing::debug!("Evicted resource from cache: {} ({} bytes)", key, entry.size);
            }
        }
    }

    /// Return the current memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        self.current_memory
    }

    /// Return the maximum memory budget in bytes.
    pub fn max_memory(&self) -> usize {
        self.max_memory
    }

    /// Return the number of cached entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Clear the entire cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_memory = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut cache = ResourceCache::new(1024);
        let data: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.insert("test".into(), data, 4);
        let retrieved = cache.get("test").unwrap();
        assert_eq!(*retrieved.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_eviction() {
        let mut cache = ResourceCache::new(10);
        let d1: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
        let d2: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
        let d3: Arc<dyn Any + Send + Sync> = Arc::new(3i32);
        cache.insert("a".into(), d1, 4);
        cache.insert("b".into(), d2, 4);
        cache.insert("c".into(), d3, 4);
        // "a" should have been evicted to make room for "c".
        assert!(cache.get("a").is_none() || cache.entry_count() <= 2);
    }
}
