//! Lua API for actor (entity) manipulation — spawning, destroying, and
//! reading / writing components.

use mlua::prelude::*;
use tracing::debug;

use crate::bindings::{LuaEntity, LuaVec2};

/// Register the `Actor` table into the Lua globals.
pub fn register(lua: &Lua) -> LuaResult<()> {
    let actor = lua.create_table()?;

    // Actor.spawn(template, x, y) -> Entity
    actor.set(
        "spawn",
        lua.create_function(|_, (template, x, y): (String, f32, f32)| {
            debug!("Lua: Actor.spawn('{template}', {x}, {y})");
            // In a live engine the spawn request would be forwarded to the ECS
            // World. For now we return a placeholder entity.
            Ok(LuaEntity { id: 0, generation: 0 })
        })?,
    )?;

    // Actor.destroy(entity)
    actor.set(
        "destroy",
        lua.create_function(|_, entity: LuaEntity| {
            debug!("Lua: Actor.destroy({:?})", entity);
            Ok(())
        })?,
    )?;

    // Actor.get_position(entity) -> Vec2
    actor.set(
        "get_position",
        lua.create_function(|_, entity: LuaEntity| {
            debug!("Lua: Actor.get_position({:?})", entity);
            Ok(LuaVec2 { x: 0.0, y: 0.0 })
        })?,
    )?;

    // Actor.set_position(entity, vec2)
    actor.set(
        "set_position",
        lua.create_function(|_, (entity, pos): (LuaEntity, LuaVec2)| {
            debug!("Lua: Actor.set_position({:?}, {:?})", entity, pos);
            Ok(())
        })?,
    )?;

    // Actor.get_health(entity) -> number
    actor.set(
        "get_health",
        lua.create_function(|_, entity: LuaEntity| {
            debug!("Lua: Actor.get_health({:?})", entity);
            Ok(100i32)
        })?,
    )?;

    // Actor.set_health(entity, hp)
    actor.set(
        "set_health",
        lua.create_function(|_, (entity, hp): (LuaEntity, i32)| {
            debug!("Lua: Actor.set_health({:?}, {hp})", entity);
            Ok(())
        })?,
    )?;

    // Actor.get_component(entity, component_name) -> table | nil
    actor.set(
        "get_component",
        lua.create_function(|_lua, (entity, name): (LuaEntity, String)| {
            debug!("Lua: Actor.get_component({:?}, '{name}')", entity);
            // Placeholder — returns nil until backed by a real ECS.
            Ok(mlua::Value::Nil)
        })?,
    )?;

    // Actor.set_component(entity, component_name, data_table)
    actor.set(
        "set_component",
        lua.create_function(|_, (entity, name, _data): (LuaEntity, String, LuaTable)| {
            debug!("Lua: Actor.set_component({:?}, '{name}')", entity);
            Ok(())
        })?,
    )?;

    // Actor.find_by_tag(tag) -> {Entity...}
    actor.set(
        "find_by_tag",
        lua.create_function(|lua, tag: String| {
            debug!("Lua: Actor.find_by_tag('{tag}')");
            let result = lua.create_table()?;
            // Placeholder — empty list.
            Ok(result)
        })?,
    )?;

    lua.globals().set("Actor", actor)?;
    Ok(())
}
