//! Lua API for UI — text, buttons, and simple HUD elements.

use mlua::prelude::*;
use tracing::debug;

use crate::bindings::LuaColor;

/// Register the `UI` table into the Lua globals.
pub fn register(lua: &Lua) -> LuaResult<()> {
    let ui = lua.create_table()?;

    // UI.show_text(text, x, y, options?)
    // options: { font_size, color, align }
    ui.set(
        "show_text",
        lua.create_function(|_, (text, x, y, opts): (String, f32, f32, Option<LuaTable>)| {
            debug!("Lua: UI.show_text('{text}', {x}, {y})");
            let _ = opts; // consumed by the real renderer
            Ok(())
        })?,
    )?;

    // UI.create_button(id, text, x, y, w, h) -> button_id
    ui.set(
        "create_button",
        lua.create_function(
            |_, (id, text, x, y, w, h): (String, String, f32, f32, f32, f32)| {
                debug!("Lua: UI.create_button('{id}', '{text}', {x},{y},{w},{h})");
                Ok(id)
            },
        )?,
    )?;

    // UI.is_button_clicked(id) -> bool
    ui.set(
        "is_button_clicked",
        lua.create_function(|_, id: String| {
            debug!("Lua: UI.is_button_clicked('{id}')");
            Ok(false)
        })?,
    )?;

    // UI.remove_element(id)
    ui.set(
        "remove_element",
        lua.create_function(|_, id: String| {
            debug!("Lua: UI.remove_element('{id}')");
            Ok(())
        })?,
    )?;

    // UI.show_image(path, x, y, w?, h?)
    ui.set(
        "show_image",
        lua.create_function(
            |_, (path, x, y, w, h): (String, f32, f32, Option<f32>, Option<f32>)| {
                debug!("Lua: UI.show_image('{path}', {x},{y})");
                let _ = (w, h);
                Ok(())
            },
        )?,
    )?;

    // UI.show_progress_bar(id, x, y, w, h, value, max_value, color?)
    ui.set(
        "show_progress_bar",
        lua.create_function(
            |_,
             (id, x, y, w, h, value, max_value, _color): (
                String,
                f32,
                f32,
                f32,
                f32,
                f32,
                f32,
                Option<LuaColor>,
            )| {
                debug!(
                    "Lua: UI.show_progress_bar('{id}', {x},{y},{w},{h}, {value}/{max_value})"
                );
                Ok(())
            },
        )?,
    )?;

    // UI.clear()
    ui.set(
        "clear",
        lua.create_function(|_, ()| {
            debug!("Lua: UI.clear()");
            Ok(())
        })?,
    )?;

    lua.globals().set("UI", ui)?;
    Ok(())
}
