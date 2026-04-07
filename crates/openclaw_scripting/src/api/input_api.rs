//! Lua API for querying input state — keyboard, mouse, and gamepad.

use mlua::prelude::*;
use tracing::debug;

use crate::bindings::LuaVec2;

/// Register the `Input` table into the Lua globals.
pub fn register(lua: &Lua) -> LuaResult<()> {
    let input = lua.create_table()?;

    // Input.is_key_pressed(key_name) -> bool
    input.set(
        "is_key_pressed",
        lua.create_function(|_, key: String| {
            debug!("Lua: Input.is_key_pressed('{key}')");
            Ok(false)
        })?,
    )?;

    // Input.is_key_just_pressed(key_name) -> bool
    input.set(
        "is_key_just_pressed",
        lua.create_function(|_, key: String| {
            debug!("Lua: Input.is_key_just_pressed('{key}')");
            Ok(false)
        })?,
    )?;

    // Input.is_key_released(key_name) -> bool
    input.set(
        "is_key_released",
        lua.create_function(|_, key: String| {
            debug!("Lua: Input.is_key_released('{key}')");
            Ok(true)
        })?,
    )?;

    // Input.get_mouse_pos() -> Vec2
    input.set(
        "get_mouse_pos",
        lua.create_function(|_, ()| {
            Ok(LuaVec2 { x: 0.0, y: 0.0 })
        })?,
    )?;

    // Input.is_mouse_button_pressed(button) -> bool
    input.set(
        "is_mouse_button_pressed",
        lua.create_function(|_, button: u32| {
            debug!("Lua: Input.is_mouse_button_pressed({button})");
            Ok(false)
        })?,
    )?;

    // Input.get_axis(axis_name) -> number  (-1..1)
    input.set(
        "get_axis",
        lua.create_function(|_, axis: String| {
            debug!("Lua: Input.get_axis('{axis}')");
            Ok(0.0f32)
        })?,
    )?;

    // Input.is_gamepad_button_pressed(pad_index, button_name) -> bool
    input.set(
        "is_gamepad_button_pressed",
        lua.create_function(|_, (pad, button): (u32, String)| {
            debug!("Lua: Input.is_gamepad_button_pressed({pad}, '{button}')");
            Ok(false)
        })?,
    )?;

    lua.globals().set("Input", input)?;
    Ok(())
}
