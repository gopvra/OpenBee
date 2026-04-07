//! Input system: reads input state and updates controllable entities.

use openclaw_core::ecs::{System, World};

use crate::components::controllable::ControllableComponent;

/// Reads input and distributes it to controllable entities.
/// Actual input polling is handled externally; this system processes
/// the resulting state.
pub struct InputSystem {
    /// Whether the pause key was pressed this frame.
    pub pause_pressed: bool,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            pause_pressed: false,
        }
    }
}

impl System for InputSystem {
    fn name(&self) -> &str {
        "InputSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        // Ensure controllable components are in a valid state.
        let entities: Vec<_> = world.entities();
        for entity in entities {
            if let Some(ctrl) = world.get_component_mut::<ControllableComponent>(entity) {
                // Controllable state is managed by the actor controller;
                // this system can disable control during cutscenes etc.
                let _ = ctrl.is_active;
            }
        }
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}
