//! Entity with generational index for safe handle reuse.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A unique entity handle using a generational index scheme.
/// The generation counter prevents use-after-free bugs when entity slots are recycled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    /// Unique index within the entity allocator.
    pub id: u32,
    /// Generation counter; incremented each time this slot is recycled.
    pub generation: u32,
}

impl Entity {
    /// Create a new entity with the given id and generation.
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}v{})", self.id, self.generation)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}v{})", self.id, self.generation)
    }
}

/// Entry in the allocator tracking whether a slot is alive and its current generation.
struct AllocatorEntry {
    is_alive: bool,
    generation: u32,
}

/// Allocates and recycles entity handles with generational indices.
pub struct EntityAllocator {
    entries: Vec<AllocatorEntry>,
    free_ids: Vec<u32>,
}

impl EntityAllocator {
    /// Create a new empty allocator.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_ids: Vec::new(),
        }
    }

    /// Allocate a new entity. Reuses freed slots when available.
    pub fn allocate(&mut self) -> Entity {
        if let Some(id) = self.free_ids.pop() {
            let entry = &mut self.entries[id as usize];
            entry.generation += 1;
            entry.is_alive = true;
            Entity::new(id, entry.generation)
        } else {
            let id = u32::try_from(self.entries.len())
                .expect("Entity limit exceeded: more than 4 billion entities");
            self.entries.push(AllocatorEntry {
                is_alive: true,
                generation: 0,
            });
            Entity::new(id, 0)
        }
    }

    /// Deallocate an entity, marking its slot as free for future reuse.
    /// Returns `true` if the entity was valid and successfully deallocated.
    pub fn deallocate(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        self.entries[entity.id as usize].is_alive = false;
        self.free_ids.push(entity.id);
        true
    }

    /// Check whether an entity handle is still valid (alive and correct generation).
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.id as usize;
        if idx >= self.entries.len() {
            return false;
        }
        let entry = &self.entries[idx];
        entry.is_alive && entry.generation == entity.generation
    }

    /// Return an iterator over all currently alive entities.
    pub fn alive_entities(&self) -> Vec<Entity> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_alive)
            .map(|(i, e)| Entity::new(i as u32, e.generation))
            .collect()
    }

    /// Return the number of currently alive entities.
    pub fn alive_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_alive).count()
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_and_deallocate() {
        let mut alloc = EntityAllocator::new();
        let e1 = alloc.allocate();
        let e2 = alloc.allocate();
        assert_eq!(e1.id, 0);
        assert_eq!(e2.id, 1);
        assert!(alloc.is_alive(e1));
        assert!(alloc.is_alive(e2));

        alloc.deallocate(e1);
        assert!(!alloc.is_alive(e1));

        let e3 = alloc.allocate();
        assert_eq!(e3.id, 0);
        assert_eq!(e3.generation, 1);
        assert!(!alloc.is_alive(e1)); // old generation
        assert!(alloc.is_alive(e3));
    }
}
