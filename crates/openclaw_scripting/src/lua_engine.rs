//! Lua VM management — loading, executing, and updating scripts.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use mlua::prelude::*;
use tracing::{debug, error, info};

use crate::bindings;

/// Central Lua scripting engine wrapping a Lua 5.4 VM.
pub struct LuaEngine {
    /// The underlying Lua state.
    lua: Lua,
    /// Map of script name to its source code.
    scripts: HashMap<String, String>,
    /// Set of script names that have already been loaded (executed) in the VM.
    loaded: HashSet<String>,
}

impl LuaEngine {
    /// Create a new Lua engine with standard libraries opened and default
    /// bindings registered.
    pub fn new() -> Result<Self> {
        let lua = Lua::new();

        // Register built-in type constructors and utility functions.
        bindings::register_all(&lua).map_err(|e| anyhow::anyhow!("Lua binding error: {e}"))?;

        info!("LuaEngine initialised with Lua 5.4");

        Ok(Self {
            lua,
            scripts: HashMap::new(),
            loaded: HashSet::new(),
        })
    }

    /// Store a script under `name`. This does **not** execute it; call
    /// [`execute`] or [`load_script`] followed by [`call_function`] for that.
    pub fn load_script(&mut self, name: &str, source: &str) -> Result<()> {
        debug!("Loading script '{name}'");
        self.scripts.insert(name.to_string(), source.to_string());

        // Compile and execute so that top-level definitions (functions, globals)
        // become available immediately.
        self.lua
            .load(source)
            .set_name(name)
            .exec()
            .map_err(|e| {
                error!("Failed to load script '{name}': {e}");
                anyhow::anyhow!("Lua load error in '{name}': {e}")
            })?;

        self.loaded.insert(name.to_string());
        Ok(())
    }

    /// Execute an arbitrary Lua chunk and return the resulting value.
    pub fn execute(&self, script: &str) -> Result<mlua::Value> {
        let val = self
            .lua
            .load(script)
            .eval::<mlua::Value>()
            .map_err(|e| anyhow::anyhow!("Lua exec error: {e}"))?;
        Ok(val)
    }

    /// Call a previously-defined global Lua function by name.
    pub fn call_function<A: IntoLuaMulti>(&self, name: &str, args: A) -> Result<()> {
        let globals = self.lua.globals();
        let func: mlua::Function = globals.get(name).map_err(|e| {
            anyhow::anyhow!("Lua function '{name}' not found: {e}")
        })?;
        func.call::<()>(args).map_err(|e| {
            error!("Error calling Lua function '{name}': {e}");
            anyhow::anyhow!("Lua call error in '{name}': {e}")
        })?;
        Ok(())
    }

    /// Register a value as a Lua global variable.
    pub fn register_global<V: IntoLua>(&self, name: &str, value: V) -> Result<()> {
        self.lua
            .globals()
            .set(name, value)
            .map_err(|e| anyhow::anyhow!("Failed to set global '{name}': {e}"))?;
        Ok(())
    }

    /// Tick the scripting engine. Calls the global `_update(dt)` function if it
    /// exists, which serves as the per-frame script hook.
    pub fn update(&self, dt: f64) -> Result<()> {
        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<mlua::Function>("_update") {
            func.call::<()>(dt).map_err(|e| {
                error!("Error in Lua _update: {e}");
                anyhow::anyhow!("Lua _update error: {e}")
            })?;
        }
        Ok(())
    }

    /// Obtain a reference to the underlying Lua state for advanced usage.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Return the names of all registered scripts.
    pub fn script_names(&self) -> Vec<&str> {
        self.scripts.keys().map(|s| s.as_str()).collect()
    }

    /// Check whether a script has been loaded and executed.
    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded.contains(name)
    }
}
