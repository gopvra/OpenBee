//! Input-related events.

use crate::events::event_bus::Event;
use crate::input::gamepad::GamepadId;
use crate::input::keyboard::KeyCode;
use crate::input::mouse::MouseButton;
use crate::input::touch::TouchId;

/// A keyboard key was pressed.
#[derive(Debug, Clone)]
pub struct KeyPressed {
    pub key: KeyCode,
}
impl Event for KeyPressed {}

/// A keyboard key was released.
#[derive(Debug, Clone)]
pub struct KeyReleased {
    pub key: KeyCode,
}
impl Event for KeyReleased {}

/// The mouse moved to a new position.
#[derive(Debug, Clone)]
pub struct MouseMoved {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
}
impl Event for MouseMoved {}

/// A mouse button was pressed.
#[derive(Debug, Clone)]
pub struct MouseButtonPressed {
    pub button: MouseButton,
    pub x: i32,
    pub y: i32,
}
impl Event for MouseButtonPressed {}

/// A mouse button was released.
#[derive(Debug, Clone)]
pub struct MouseButtonReleased {
    pub button: MouseButton,
    pub x: i32,
    pub y: i32,
}
impl Event for MouseButtonReleased {}

/// A gamepad was connected.
#[derive(Debug, Clone)]
pub struct GamepadConnected {
    pub gamepad_id: GamepadId,
    pub name: String,
}
impl Event for GamepadConnected {}

/// A gamepad was disconnected.
#[derive(Debug, Clone)]
pub struct GamepadDisconnected {
    pub gamepad_id: GamepadId,
}
impl Event for GamepadDisconnected {}

/// A touch began (finger down).
#[derive(Debug, Clone)]
pub struct TouchBegan {
    pub touch_id: TouchId,
    pub x: f32,
    pub y: f32,
}
impl Event for TouchBegan {}

/// A touch moved (finger dragged).
#[derive(Debug, Clone)]
pub struct TouchMoved {
    pub touch_id: TouchId,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
}
impl Event for TouchMoved {}

/// A touch ended (finger up).
#[derive(Debug, Clone)]
pub struct TouchEnded {
    pub touch_id: TouchId,
    pub x: f32,
    pub y: f32,
}
impl Event for TouchEnded {}
