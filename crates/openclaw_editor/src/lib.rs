//! # OpenClaw Editor
//!
//! Visual level editor for OpenClaw built on egui. Provides tile painting, actor
//! placement, property inspection, undo/redo, and a live preview renderer.

pub mod actor_editor;
pub mod editor_app;
pub mod editor_ui;
pub mod preview;
pub mod property_panel;
pub mod tile_editor;

pub use editor_app::{EditorApp, EditorState, EditorTool};
