//! Collision events and contact listener.

use crate::ecs::Entity;

/// The type of collision event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionType {
    /// Two bodies started touching.
    Begin,
    /// Two bodies stopped touching.
    End,
    /// A sensor overlap occurred.
    Sensor,
}

/// A collision event between two entities.
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    /// First entity involved.
    pub entity_a: Entity,
    /// Second entity involved.
    pub entity_b: Entity,
    /// Type of collision.
    pub collision_type: CollisionType,
}

/// Collects collision events each physics step for later processing.
pub struct ContactListener {
    events: Vec<CollisionEvent>,
}

impl ContactListener {
    /// Create a new empty listener.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record a collision event.
    pub fn push_event(&mut self, event: CollisionEvent) {
        self.events.push(event);
    }

    /// Drain all collected events from the current frame.
    pub fn drain_events(&mut self) -> Vec<CollisionEvent> {
        std::mem::take(&mut self.events)
    }

    /// Return the number of pending events.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Clear all pending events without processing them.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for ContactListener {
    fn default() -> Self {
        Self::new()
    }
}
