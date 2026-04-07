//! Lua API modules — each sub-module registers a table of functions into the
//! Lua global namespace under a well-known name (e.g. `Actor`, `Physics`).

pub mod actor_api;
pub mod audio_api;
pub mod input_api;
pub mod physics_api;
pub mod ui_api;

use mlua::prelude::*;

/// Register every API module into the given Lua state.
pub fn register_all_apis(lua: &Lua) -> LuaResult<()> {
    actor_api::register(lua)?;
    physics_api::register(lua)?;
    audio_api::register(lua)?;
    input_api::register(lua)?;
    ui_api::register(lua)?;
    Ok(())
}
