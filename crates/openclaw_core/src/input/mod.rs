//! Input handling: keyboard, mouse, gamepad, and touch.

pub mod gamepad;
pub mod input_manager;
pub mod keybindings;
pub mod keyboard;
pub mod mouse;
pub mod touch;

pub use gamepad::{GamepadAxis, GamepadButton, GamepadId, GamepadManager, GamepadState};
pub use input_manager::InputManager;
pub use keybindings::{InputSource, KeyBinding, KeyBindingMap};
pub use keyboard::KeyCode;
pub use mouse::{MouseButton, MouseState};
pub use touch::{TouchGesture, TouchId, TouchPoint};
