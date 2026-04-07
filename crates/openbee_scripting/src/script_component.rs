//! Script component that can be attached to entities so that Lua scripts drive
//! their behaviour each frame.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A value that can be exchanged between Rust and a Lua script property table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptValue {
    /// 64-bit integer.
    Int(i64),
    /// 64-bit float.
    Float(f64),
    /// UTF-8 string.
    String(String),
    /// Boolean.
    Bool(bool),
    /// 2-D vector stored as two `f32` components.
    Vec2(f32, f32),
}

impl std::fmt::Display for ScriptValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "\"{v}\""),
            Self::Bool(v) => write!(f, "{v}"),
            Self::Vec2(x, y) => write!(f, "Vec2({x}, {y})"),
        }
    }
}

/// Component that binds a Lua script to an entity.
///
/// Each frame the scripting system will call the script's `on_update(entity, dt)`
/// callback (if defined), passing the owning entity handle and delta time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptComponent {
    /// Name of the script (must match a script registered with [`LuaEngine`]).
    pub script_name: String,
    /// Whether the script is currently active.
    pub enabled: bool,
    /// Arbitrary key/value properties exposed to the script.
    pub properties: HashMap<String, ScriptValue>,
}

impl ScriptComponent {
    /// Create a new enabled `ScriptComponent` with the given script name and no
    /// properties.
    pub fn new(script_name: impl Into<String>) -> Self {
        Self {
            script_name: script_name.into(),
            enabled: true,
            properties: HashMap::new(),
        }
    }

    /// Set a property value, returning `&mut self` for chaining.
    pub fn with_property(mut self, key: impl Into<String>, value: ScriptValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    /// Retrieve a property by key.
    pub fn get_property(&self, key: &str) -> Option<&ScriptValue> {
        self.properties.get(key)
    }

    /// Set or overwrite a property.
    pub fn set_property(&mut self, key: impl Into<String>, value: ScriptValue) {
        self.properties.insert(key.into(), value);
    }
}
