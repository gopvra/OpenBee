//! Component storage using type-erased maps keyed by TypeId.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::entity::Entity;

/// Marker trait for all components. Components must be `'static` so they can be stored as `dyn Any`.
pub trait Component: Any + Send + Sync + 'static {}

/// Stores all component data for all entities, organized by component type.
/// Each component type has its own sub-map from entity id to the component value.
pub struct ComponentStorage {
    /// Outer key: TypeId of the component type.
    /// Inner key: entity id (u32).
    stores: HashMap<TypeId, HashMap<u32, Box<dyn Any + Send + Sync>>>,
}

impl ComponentStorage {
    /// Create empty component storage.
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    /// Insert a component for the given entity. Overwrites any previous value of the same type.
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let store = self.stores.entry(type_id).or_insert_with(HashMap::new);
        store.insert(entity.id, Box::new(component));
    }

    /// Remove a component of the given type from an entity. Returns the removed value if it existed.
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let store = self.stores.get_mut(&type_id)?;
        let boxed = store.remove(&entity.id)?;
        boxed.downcast::<T>().ok().map(|b| *b)
    }

    /// Get a shared reference to a component on an entity.
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let store = self.stores.get(&type_id)?;
        let boxed = store.get(&entity.id)?;
        boxed.downcast_ref::<T>()
    }

    /// Get a mutable reference to a component on an entity.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let store = self.stores.get_mut(&type_id)?;
        let boxed = store.get_mut(&entity.id)?;
        boxed.downcast_mut::<T>()
    }

    /// Check whether an entity has a component of the given type.
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.stores
            .get(&type_id)
            .map_or(false, |store| store.contains_key(&entity.id))
    }

    /// Return all entity ids that have a component of the given type.
    pub fn entities_with<T: Component>(&self) -> Vec<u32> {
        let type_id = TypeId::of::<T>();
        self.stores
            .get(&type_id)
            .map(|store| store.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Remove all components associated with a given entity.
    pub fn remove_all(&mut self, entity: Entity) {
        for store in self.stores.values_mut() {
            store.remove(&entity.id);
        }
    }

    /// Clear all component data.
    pub fn clear(&mut self) {
        self.stores.clear();
    }
}

impl Default for ComponentStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }
    impl Component for Velocity {}

    #[test]
    fn test_insert_and_get() {
        let mut storage = ComponentStorage::new();
        let entity = Entity::new(0, 0);
        storage.insert(entity, Position { x: 1.0, y: 2.0 });
        let pos = storage.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn test_remove() {
        let mut storage = ComponentStorage::new();
        let entity = Entity::new(0, 0);
        storage.insert(entity, Position { x: 1.0, y: 2.0 });
        let removed = storage.remove::<Position>(entity).unwrap();
        assert_eq!(removed.x, 1.0);
        assert!(!storage.has::<Position>(entity));
    }
}
