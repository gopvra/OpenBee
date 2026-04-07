//! World struct that ties together entities, components, and systems.

use super::component::{Component, ComponentStorage};
use super::entity::{Entity, EntityAllocator};
use super::system::SystemScheduler;

/// The central data store for the ECS. Holds all entities, their components,
/// and the system scheduler.
pub struct World {
    allocator: EntityAllocator,
    components: ComponentStorage,
    scheduler: SystemScheduler,
}

impl World {
    /// Create a new empty world.
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            components: ComponentStorage::new(),
            scheduler: SystemScheduler::new(),
        }
    }

    /// Create a new entity and return its handle.
    pub fn create_entity(&mut self) -> Entity {
        self.allocator.allocate()
    }

    /// Destroy an entity and remove all of its components.
    /// Returns `true` if the entity was valid and destroyed.
    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        if self.allocator.deallocate(entity) {
            self.components.remove_all(entity);
            true
        } else {
            false
        }
    }

    /// Check whether an entity handle is still valid.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.allocator.is_alive(entity)
    }

    /// Add a component to an entity. Overwrites any previous component of the same type.
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        debug_assert!(
            self.is_alive(entity),
            "Cannot add component to dead entity {entity}"
        );
        self.components.insert(entity, component);
    }

    /// Remove a component from an entity and return it.
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        self.components.remove::<T>(entity)
    }

    /// Get a shared reference to a component on an entity.
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.components.get::<T>(entity)
    }

    /// Get a mutable reference to a component on an entity.
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut::<T>(entity)
    }

    /// Check whether an entity has a component of the given type.
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        self.components.has::<T>(entity)
    }

    /// Return all alive entities.
    pub fn entities(&self) -> Vec<Entity> {
        self.allocator.alive_entities()
    }

    /// Return the number of alive entities.
    pub fn entity_count(&self) -> usize {
        self.allocator.alive_count()
    }

    /// Get a mutable reference to the system scheduler.
    pub fn scheduler_mut(&mut self) -> &mut SystemScheduler {
        &mut self.scheduler
    }

    /// Run all registered systems with the given delta time.
    /// Note: we temporarily take the scheduler out to avoid borrow issues.
    pub fn update(&mut self, dt: f32) {
        let mut scheduler = std::mem::take(&mut self.scheduler);
        scheduler.update_all(self, dt);
        self.scheduler = scheduler;
    }

    /// Get a shared reference to the component storage.
    pub fn components(&self) -> &ComponentStorage {
        &self.components
    }

    /// Get a mutable reference to the component storage.
    pub fn components_mut(&mut self) -> &mut ComponentStorage {
        &mut self.components
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::Component;

    #[derive(Debug, PartialEq)]
    struct Health(i32);
    impl Component for Health {}

    #[test]
    fn test_world_basic() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(e, Health(100));
        assert_eq!(world.get_component::<Health>(e).unwrap().0, 100);
        world.destroy_entity(e);
        assert!(!world.is_alive(e));
    }
}
