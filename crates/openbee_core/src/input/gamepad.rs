//! Gamepad (controller) input types and manager using gilrs.

use gilrs::{Gilrs, Event as GilrsEvent, EventType as GilrsEventType};
use std::collections::HashMap;

/// Unique identifier for a connected gamepad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GamepadId(pub usize);

/// Standard gamepad button names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum GamepadButton {
    South,      // A / Cross
    East,       // B / Circle
    West,       // X / Square
    North,      // Y / Triangle
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Select,
    Start,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Guide,
}

/// Standard gamepad axis names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

/// State snapshot of a single gamepad.
#[derive(Debug, Clone)]
pub struct GamepadState {
    /// Currently pressed buttons.
    pub buttons: Vec<GamepadButton>,
    /// Axis values keyed by axis (-1.0..=1.0).
    pub axes: HashMap<GamepadAxis, f32>,
    /// Whether this gamepad is currently connected.
    pub connected: bool,
}

impl GamepadState {
    /// Create a new default gamepad state.
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            axes: HashMap::new(),
            connected: false,
        }
    }

    /// Check if a button is currently pressed.
    pub fn is_pressed(&self, button: GamepadButton) -> bool {
        self.buttons.contains(&button)
    }

    /// Get the value of an axis (0.0 if not present).
    pub fn axis_value(&self, axis: GamepadAxis) -> f32 {
        self.axes.get(&axis).copied().unwrap_or(0.0)
    }
}

impl Default for GamepadState {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages gamepad connections and state using gilrs.
pub struct GamepadManager {
    gilrs: Gilrs,
    states: HashMap<GamepadId, GamepadState>,
}

impl GamepadManager {
    /// Create a new gamepad manager, initializing the gilrs backend.
    pub fn new() -> Result<Self, String> {
        let gilrs = Gilrs::new().map_err(|e| format!("Failed to initialize gilrs: {}", e))?;
        let mut manager = Self {
            gilrs,
            states: HashMap::new(),
        };
        manager.scan_connected();
        Ok(manager)
    }

    /// Scan for initially connected gamepads.
    fn scan_connected(&mut self) {
        for (id, gamepad) in self.gilrs.gamepads() {
            if gamepad.is_connected() {
                let gid = GamepadId(id.into());
                let mut state = GamepadState::new();
                state.connected = true;
                self.states.insert(gid, state);
                tracing::info!("Gamepad connected: {:?} - {}", gid, gamepad.name());
            }
        }
    }

    /// Poll for gamepad events and update internal state. Call once per frame.
    pub fn update(&mut self) {
        while let Some(GilrsEvent { id, event, .. }) = self.gilrs.next_event() {
            let gid = GamepadId(id.into());
            match event {
                GilrsEventType::Connected => {
                    let mut state = GamepadState::new();
                    state.connected = true;
                    self.states.insert(gid, state);
                    tracing::info!("Gamepad connected: {:?}", gid);
                }
                GilrsEventType::Disconnected => {
                    if let Some(state) = self.states.get_mut(&gid) {
                        state.connected = false;
                        state.buttons.clear();
                        state.axes.clear();
                    }
                    tracing::info!("Gamepad disconnected: {:?}", gid);
                }
                GilrsEventType::ButtonPressed(button, _) => {
                    if let (Some(state), Some(mapped)) =
                        (self.states.get_mut(&gid), map_gilrs_button(button))
                    {
                        if !state.buttons.contains(&mapped) {
                            state.buttons.push(mapped);
                        }
                    }
                }
                GilrsEventType::ButtonReleased(button, _) => {
                    if let (Some(state), Some(mapped)) =
                        (self.states.get_mut(&gid), map_gilrs_button(button))
                    {
                        state.buttons.retain(|b| *b != mapped);
                    }
                }
                GilrsEventType::AxisChanged(axis, value, _) => {
                    if let (Some(state), Some(mapped)) =
                        (self.states.get_mut(&gid), map_gilrs_axis(axis))
                    {
                        state.axes.insert(mapped, value);
                    }
                }
                _ => {}
            }
        }
    }

    /// Get the state of a specific gamepad.
    pub fn get_state(&self, id: GamepadId) -> Option<&GamepadState> {
        self.states.get(&id)
    }

    /// Get all connected gamepad IDs.
    pub fn connected_gamepads(&self) -> Vec<GamepadId> {
        self.states
            .iter()
            .filter(|(_, s)| s.connected)
            .map(|(id, _)| *id)
            .collect()
    }
}

/// Map a gilrs button to our GamepadButton enum.
fn map_gilrs_button(button: gilrs::Button) -> Option<GamepadButton> {
    match button {
        gilrs::Button::South => Some(GamepadButton::South),
        gilrs::Button::East => Some(GamepadButton::East),
        gilrs::Button::West => Some(GamepadButton::West),
        gilrs::Button::North => Some(GamepadButton::North),
        gilrs::Button::LeftTrigger => Some(GamepadButton::LeftBumper),
        gilrs::Button::RightTrigger => Some(GamepadButton::RightBumper),
        gilrs::Button::LeftTrigger2 => Some(GamepadButton::LeftTrigger),
        gilrs::Button::RightTrigger2 => Some(GamepadButton::RightTrigger),
        gilrs::Button::Select => Some(GamepadButton::Select),
        gilrs::Button::Start => Some(GamepadButton::Start),
        gilrs::Button::LeftThumb => Some(GamepadButton::LeftStick),
        gilrs::Button::RightThumb => Some(GamepadButton::RightStick),
        gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
        gilrs::Button::Mode => Some(GamepadButton::Guide),
        _ => None,
    }
}

/// Map a gilrs axis to our GamepadAxis enum.
fn map_gilrs_axis(axis: gilrs::Axis) -> Option<GamepadAxis> {
    match axis {
        gilrs::Axis::LeftStickX => Some(GamepadAxis::LeftStickX),
        gilrs::Axis::LeftStickY => Some(GamepadAxis::LeftStickY),
        gilrs::Axis::RightStickX => Some(GamepadAxis::RightStickX),
        gilrs::Axis::RightStickY => Some(GamepadAxis::RightStickY),
        _ => None,
    }
}
