//! # OpenBee Scripting
//!
//! Lua 5.4 scripting integration for OpenBee. Provides a Lua VM, Rust-to-Lua
//! bindings for core engine types, a `ScriptComponent` for attaching scripts to
//! entities, and a rich API surface for actor manipulation, physics, audio,
//! input, and UI.

pub mod api;
pub mod bindings;
pub mod lua_engine;
pub mod script_component;

pub use lua_engine::LuaEngine;
pub use script_component::{ScriptComponent, ScriptValue};
