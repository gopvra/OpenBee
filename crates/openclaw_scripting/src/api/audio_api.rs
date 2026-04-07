//! Lua API for audio — sound effects, music playback, and volume control.

use mlua::prelude::*;
use tracing::debug;

/// Register the `Audio` table into the Lua globals.
pub fn register(lua: &Lua) -> LuaResult<()> {
    let audio = lua.create_table()?;

    // Audio.play_sound(name, volume?, loop?)
    audio.set(
        "play_sound",
        lua.create_function(|_, (name, volume, looping): (String, Option<f32>, Option<bool>)| {
            let vol = volume.unwrap_or(1.0);
            let lp = looping.unwrap_or(false);
            debug!("Lua: Audio.play_sound('{name}', vol={vol}, loop={lp})");
            // Return a handle ID that can be used to stop the sound later.
            Ok(0u32)
        })?,
    )?;

    // Audio.play_music(name, volume?, fade_in_ms?)
    audio.set(
        "play_music",
        lua.create_function(
            |_, (name, volume, fade_in): (String, Option<f32>, Option<u32>)| {
                let vol = volume.unwrap_or(1.0);
                let fade = fade_in.unwrap_or(0);
                debug!("Lua: Audio.play_music('{name}', vol={vol}, fade_in={fade}ms)");
                Ok(())
            },
        )?,
    )?;

    // Audio.stop(handle)
    audio.set(
        "stop",
        lua.create_function(|_, handle: u32| {
            debug!("Lua: Audio.stop(handle={handle})");
            Ok(())
        })?,
    )?;

    // Audio.stop_music(fade_out_ms?)
    audio.set(
        "stop_music",
        lua.create_function(|_, fade_out: Option<u32>| {
            let fade = fade_out.unwrap_or(0);
            debug!("Lua: Audio.stop_music(fade_out={fade}ms)");
            Ok(())
        })?,
    )?;

    // Audio.set_volume(handle, volume)
    audio.set(
        "set_volume",
        lua.create_function(|_, (handle, volume): (u32, f32)| {
            debug!("Lua: Audio.set_volume(handle={handle}, vol={volume})");
            Ok(())
        })?,
    )?;

    // Audio.set_music_volume(volume)
    audio.set(
        "set_music_volume",
        lua.create_function(|_, volume: f32| {
            debug!("Lua: Audio.set_music_volume({volume})");
            Ok(())
        })?,
    )?;

    // Audio.set_master_volume(volume)
    audio.set(
        "set_master_volume",
        lua.create_function(|_, volume: f32| {
            debug!("Lua: Audio.set_master_volume({volume})");
            Ok(())
        })?,
    )?;

    lua.globals().set("Audio", audio)?;
    Ok(())
}
