//! Mouse button and state types.

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Extra1,
    Extra2,
}

/// Snapshot of the mouse state for a single frame.
#[derive(Debug, Clone)]
pub struct MouseState {
    /// Horizontal position in screen pixels.
    pub x: i32,
    /// Vertical position in screen pixels.
    pub y: i32,
    /// Set of currently pressed buttons.
    pub buttons: Vec<MouseButton>,
}

impl MouseState {
    /// Create a new default mouse state.
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            buttons: Vec::new(),
        }
    }

    /// Check if a button is currently pressed.
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.buttons.contains(&button)
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}
