//! Central input manager tracking key/button states across frames.

use std::collections::HashSet;

use super::keybindings::{InputSource, KeyBindingMap};
use super::keyboard::KeyCode;
use super::mouse::{MouseButton, MouseState};

/// Manages keyboard and mouse input state, tracking pressed/just-pressed/just-released transitions.
pub struct InputManager {
    /// Keys that are currently held down.
    keys_current: HashSet<KeyCode>,
    /// Keys that were held down last frame.
    keys_previous: HashSet<KeyCode>,

    /// Mouse buttons currently held down.
    mouse_current: HashSet<MouseButton>,
    /// Mouse buttons held down last frame.
    mouse_previous: HashSet<MouseButton>,

    /// Current mouse position.
    mouse_x: i32,
    mouse_y: i32,

    /// Key bindings for action-based queries.
    bindings: KeyBindingMap,
}

impl InputManager {
    /// Create a new input manager with default (empty) bindings.
    pub fn new() -> Self {
        Self {
            keys_current: HashSet::new(),
            keys_previous: HashSet::new(),
            mouse_current: HashSet::new(),
            mouse_previous: HashSet::new(),
            mouse_x: 0,
            mouse_y: 0,
            bindings: KeyBindingMap::new(),
        }
    }

    /// Create an input manager with the default Captain Claw bindings.
    pub fn with_default_bindings() -> Self {
        let mut mgr = Self::new();
        mgr.bindings = KeyBindingMap::default_bindings();
        mgr
    }

    /// Call at the start of each frame to advance the previous-state buffer.
    pub fn update(&mut self) {
        self.keys_previous = self.keys_current.clone();
        self.mouse_previous = self.mouse_current.clone();
    }

    /// Record a key press event.
    pub fn key_down(&mut self, key: KeyCode) {
        self.keys_current.insert(key);
    }

    /// Record a key release event.
    pub fn key_up(&mut self, key: KeyCode) {
        self.keys_current.remove(&key);
    }

    /// Record a mouse button press event.
    pub fn mouse_button_down(&mut self, button: MouseButton) {
        self.mouse_current.insert(button);
    }

    /// Record a mouse button release event.
    pub fn mouse_button_up(&mut self, button: MouseButton) {
        self.mouse_current.remove(&button);
    }

    /// Update the current mouse position.
    pub fn set_mouse_position(&mut self, x: i32, y: i32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    // --- Query methods ---

    /// Check if a key is currently held down.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_current.contains(&key)
    }

    /// Check if a key was pressed this frame (down now, not down last frame).
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_current.contains(&key) && !self.keys_previous.contains(&key)
    }

    /// Check if a key was released this frame (not down now, was down last frame).
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        !self.keys_current.contains(&key) && self.keys_previous.contains(&key)
    }

    /// Check if a mouse button is currently held down.
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_current.contains(&button)
    }

    /// Check if a mouse button was pressed this frame.
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_current.contains(&button) && !self.mouse_previous.contains(&button)
    }

    /// Check if a mouse button was released this frame.
    pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
        !self.mouse_current.contains(&button) && self.mouse_previous.contains(&button)
    }

    /// Get the current mouse position.
    pub fn mouse_position(&self) -> (i32, i32) {
        (self.mouse_x, self.mouse_y)
    }

    /// Get a snapshot of the current mouse state.
    pub fn mouse_state(&self) -> MouseState {
        MouseState {
            x: self.mouse_x,
            y: self.mouse_y,
            buttons: self.mouse_current.iter().copied().collect(),
        }
    }

    /// Check if any input source bound to the named action is currently pressed.
    pub fn is_action_pressed(&self, action: &str) -> bool {
        for source in self.bindings.get_bindings(action) {
            match source {
                InputSource::Key(key) => {
                    if self.is_key_pressed(*key) {
                        return true;
                    }
                }
                InputSource::Mouse(button) => {
                    if self.is_mouse_pressed(*button) {
                        return true;
                    }
                }
                _ => {
                    // Gamepad sources are handled via GamepadManager; not checked here.
                }
            }
        }
        false
    }

    /// Check if any input source bound to the named action was just pressed this frame.
    pub fn is_action_just_pressed(&self, action: &str) -> bool {
        for source in self.bindings.get_bindings(action) {
            match source {
                InputSource::Key(key) => {
                    if self.is_key_just_pressed(*key) {
                        return true;
                    }
                }
                InputSource::Mouse(button) => {
                    if self.is_mouse_just_pressed(*button) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Get a mutable reference to the key bindings.
    pub fn bindings_mut(&mut self) -> &mut KeyBindingMap {
        &mut self.bindings
    }

    /// Get a shared reference to the key bindings.
    pub fn bindings(&self) -> &KeyBindingMap {
        &self.bindings
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_just_pressed() {
        let mut input = InputManager::new();
        input.update(); // frame 0
        input.key_down(KeyCode::Space);
        assert!(input.is_key_just_pressed(KeyCode::Space));
        input.update(); // frame 1
        assert!(!input.is_key_just_pressed(KeyCode::Space));
        assert!(input.is_key_pressed(KeyCode::Space));
    }

    #[test]
    fn test_just_released() {
        let mut input = InputManager::new();
        input.key_down(KeyCode::A);
        input.update();
        input.key_up(KeyCode::A);
        assert!(input.is_key_just_released(KeyCode::A));
    }
}
