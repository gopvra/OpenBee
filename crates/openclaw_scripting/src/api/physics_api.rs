//! Lua API for physics operations — forces, raycasts, velocity.

use mlua::prelude::*;
use tracing::debug;

use crate::bindings::{LuaEntity, LuaVec2};

/// Register the `Physics` table into the Lua globals.
pub fn register(lua: &Lua) -> LuaResult<()> {
    let physics = lua.create_table()?;

    // Physics.apply_force(entity, force_x, force_y)
    physics.set(
        "apply_force",
        lua.create_function(|_, (entity, fx, fy): (LuaEntity, f32, f32)| {
            debug!("Lua: Physics.apply_force({:?}, {fx}, {fy})", entity);
            Ok(())
        })?,
    )?;

    // Physics.apply_impulse(entity, ix, iy)
    physics.set(
        "apply_impulse",
        lua.create_function(|_, (entity, ix, iy): (LuaEntity, f32, f32)| {
            debug!("Lua: Physics.apply_impulse({:?}, {ix}, {iy})", entity);
            Ok(())
        })?,
    )?;

    // Physics.set_velocity(entity, vx, vy)
    physics.set(
        "set_velocity",
        lua.create_function(|_, (entity, vx, vy): (LuaEntity, f32, f32)| {
            debug!("Lua: Physics.set_velocity({:?}, {vx}, {vy})", entity);
            Ok(())
        })?,
    )?;

    // Physics.get_velocity(entity) -> Vec2
    physics.set(
        "get_velocity",
        lua.create_function(|_, entity: LuaEntity| {
            debug!("Lua: Physics.get_velocity({:?})", entity);
            Ok(LuaVec2 { x: 0.0, y: 0.0 })
        })?,
    )?;

    // Physics.raycast(origin_x, origin_y, dir_x, dir_y, max_dist) -> { hit, point, entity }
    physics.set(
        "raycast",
        lua.create_function(
            |lua, (ox, oy, dx, dy, max_dist): (f32, f32, f32, f32, f32)| {
                debug!("Lua: Physics.raycast(({ox},{oy}), ({dx},{dy}), {max_dist})");
                let result = lua.create_table()?;
                result.set("hit", false)?;
                Ok(result)
            },
        )?,
    )?;

    // Physics.set_gravity(gx, gy)
    physics.set(
        "set_gravity",
        lua.create_function(|_, (gx, gy): (f32, f32)| {
            debug!("Lua: Physics.set_gravity({gx}, {gy})");
            Ok(())
        })?,
    )?;

    // Physics.get_gravity() -> Vec2
    physics.set(
        "get_gravity",
        lua.create_function(|_, ()| {
            Ok(LuaVec2 { x: 0.0, y: -9.81 })
        })?,
    )?;

    // Physics.set_body_type(entity, type_str)
    physics.set(
        "set_body_type",
        lua.create_function(|_, (entity, body_type): (LuaEntity, String)| {
            debug!("Lua: Physics.set_body_type({:?}, '{body_type}')", entity);
            Ok(())
        })?,
    )?;

    lua.globals().set("Physics", physics)?;
    Ok(())
}
