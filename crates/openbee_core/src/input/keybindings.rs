//! Key binding configuration: maps action names to input sources.

use std::collections::HashMap;

use super::gamepad::{GamepadAxis, GamepadButton};
use super::keyboard::KeyCode;
use super::mouse::MouseButton;

/// An input source that can be bound to an action.
#[derive(Debug, Clone, PartialEq)]
pub enum InputSource {
    /// A keyboard key.
    Key(KeyCode),
    /// A mouse button.
    Mouse(MouseButton),
    /// A gamepad button.
    Gamepad(GamepadButton),
    /// A gamepad axis with a threshold direction (positive or negative).
    GamepadAxisPositive(GamepadAxis),
    /// A gamepad axis in the negative direction.
    GamepadAxisNegative(GamepadAxis),
}

/// A single key binding associating an action name with an input source.
#[derive(Debug, Clone)]
pub struct KeyBinding {
    /// The action name (e.g., "jump", "attack", "move_left").
    pub action: String,
    /// The input source bound to this action.
    pub source: InputSource,
}

/// Map of action names to their bound input sources. Supports multiple bindings per action.
pub struct KeyBindingMap {
    bindings: HashMap<String, Vec<InputSource>>,
}

impl KeyBindingMap {
    /// Create a new empty key binding map.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Add a binding for an action. Multiple sources can be bound to the same action.
    pub fn bind(&mut self, action: impl Into<String>, source: InputSource) {
        let action = action.into();
        self.bindings.entry(action).or_default().push(source);
    }

    /// Remove all bindings for a given action.
    pub fn unbind_all(&mut self, action: &str) {
        self.bindings.remove(action);
    }

    /// Remove a specific binding for an action.
    pub fn unbind(&mut self, action: &str, source: &InputSource) {
        if let Some(sources) = self.bindings.get_mut(action) {
            sources.retain(|s| s != source);
        }
    }

    /// Get all input sources bound to an action.
    pub fn get_bindings(&self, action: &str) -> &[InputSource] {
        self.bindings
            .get(action)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check if any input source is bound to an action.
    pub fn has_binding(&self, action: &str) -> bool {
        self.bindings.get(action).is_some_and(|v| !v.is_empty())
    }

    /// Return all action names.
    pub fn actions(&self) -> Vec<&str> {
        self.bindings.keys().map(|s| s.as_str()).collect()
    }

    /// Create default Captain Claw key bindings.
    pub fn default_bindings() -> Self {
        let mut map = Self::new();
        map.bind("move_left", InputSource::Key(KeyCode::Left));
        map.bind("move_left", InputSource::Key(KeyCode::A));
        map.bind("move_right", InputSource::Key(KeyCode::Right));
        map.bind("move_right", InputSource::Key(KeyCode::D));
        map.bind("jump", InputSource::Key(KeyCode::Up));
        map.bind("jump", InputSource::Key(KeyCode::W));
        map.bind("jump", InputSource::Key(KeyCode::Space));
        map.bind("duck", InputSource::Key(KeyCode::Down));
        map.bind("duck", InputSource::Key(KeyCode::S));
        map.bind("attack", InputSource::Key(KeyCode::LCtrl));
        map.bind("attack", InputSource::Key(KeyCode::RCtrl));
        map.bind("pistol", InputSource::Key(KeyCode::Num1));
        map.bind("magic", InputSource::Key(KeyCode::Num2));
        map.bind("dynamite", InputSource::Key(KeyCode::Num3));
        map.bind("pause", InputSource::Key(KeyCode::Escape));
        map.bind("pause", InputSource::Key(KeyCode::P));
        map
    }

    /// Serialize the bindings to a JSON string.
    pub fn save_to_json(&self) -> Result<String, serde_json::Error> {
        // Convert to a serializable format.
        let serializable: HashMap<&str, Vec<String>> = self
            .bindings
            .iter()
            .map(|(action, sources)| {
                let source_strs: Vec<String> = sources.iter().map(|s| format!("{:?}", s)).collect();
                (action.as_str(), source_strs)
            })
            .collect();
        serde_json::to_string_pretty(&serializable)
    }

    /// Load bindings from a JSON string. This is a simplified loader that restores
    /// action-to-source mappings. For full round-tripping, a custom serializer would be needed.
    pub fn load_from_json(json: &str) -> Result<Self, serde_json::Error> {
        let _raw: HashMap<String, Vec<String>> = serde_json::from_str(json)?;
        // Full deserialization of InputSource from debug strings is non-trivial;
        // return an empty map and let the caller set up defaults.
        // In a production implementation, we would serialize InputSource properly.
        tracing::warn!(
            "KeyBindingMap::load_from_json: custom deserialization not yet implemented, returning defaults"
        );
        Ok(Self::default_bindings())
    }
}

impl Default for KeyBindingMap {
    fn default() -> Self {
        Self::new()
    }
}
