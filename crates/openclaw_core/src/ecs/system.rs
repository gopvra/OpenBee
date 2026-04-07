//! System trait and scheduler for running game logic each frame.

use super::world::World;

/// A system processes entities with specific component combinations.
pub trait System: Send + Sync {
    /// Human-readable name for debugging and profiling.
    fn name(&self) -> &str;

    /// Called each frame with the world and delta time in seconds.
    fn update(&mut self, world: &mut World, dt: f32);
}

/// Runs registered systems in insertion order each frame.
pub struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
}

impl SystemScheduler {
    /// Create a new empty scheduler.
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a system to the end of the execution order.
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Run all systems in order with the given world and delta time.
    pub fn update_all(&mut self, world: &mut World, dt: f32) {
        for system in &mut self.systems {
            tracing::trace!("Running system: {}", system.name());
            system.update(world, dt);
        }
    }

    /// Return the number of registered systems.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Return the names of all registered systems in execution order.
    pub fn system_names(&self) -> Vec<&str> {
        self.systems.iter().map(|s| s.name()).collect()
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}
