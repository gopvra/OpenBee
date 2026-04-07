//! Core editor application state, undo/redo, and document management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Level data structures
// ---------------------------------------------------------------------------

/// Lightweight tile-map representation used by the editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLayer {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// Row-major tile IDs; `0` means empty.
    pub tiles: Vec<u32>,
    pub visible: bool,
}

impl TileLayer {
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            tiles: vec![0; (width * height) as usize],
            visible: true,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<u32> {
        if x < self.width && y < self.height {
            Some(self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn set(&mut self, x: u32, y: u32, tile_id: u32) {
        if x < self.width && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile_id;
        }
    }
}

/// An actor instance placed in the level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorInstance {
    pub id: u64,
    pub template: String,
    pub x: f32,
    pub y: f32,
    pub properties: HashMap<String, String>,
}

/// All editable data for a single level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelEditorData {
    pub tile_layers: Vec<TileLayer>,
    pub actors: Vec<ActorInstance>,
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
}

impl Default for LevelEditorData {
    fn default() -> Self {
        Self {
            tile_layers: vec![TileLayer::new("Background", 100, 100)],
            actors: Vec::new(),
            width: 100,
            height: 100,
            tile_size: 64,
        }
    }
}

// ---------------------------------------------------------------------------
// Document wrapper
// ---------------------------------------------------------------------------

/// A level document open in the editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelDocument {
    /// Human-readable name.
    pub name: String,
    /// File path on disk, if saved.
    pub path: Option<String>,
    /// Whether the document has unsaved changes.
    pub modified: bool,
    /// The actual level data being edited.
    pub level_data: LevelEditorData,
}

impl LevelDocument {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: None,
            modified: false,
            level_data: LevelEditorData::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Undo / Redo
// ---------------------------------------------------------------------------

/// The kind of mutation that was applied to the level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    PlaceTile {
        x: u32,
        y: u32,
        old: Option<u32>,
        new: Option<u32>,
    },
    PlaceActor {
        id: u64,
        template: String,
        x: f32,
        y: f32,
    },
    DeleteActor {
        id: u64,
    },
    MoveActor {
        id: u64,
        old_pos: (f32, f32),
        new_pos: (f32, f32),
    },
    ModifyProperty {
        entity_id: u64,
        property: String,
        old_value: String,
        new_value: String,
    },
}

/// A recorded editor action for undo/redo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorAction {
    pub action_type: ActionType,
    pub timestamp: f64,
}

// ---------------------------------------------------------------------------
// Tools & state
// ---------------------------------------------------------------------------

/// Which tool is currently active in the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorTool {
    Select,
    TileBrush,
    TileEraser,
    ActorPlace,
    TriggerZone,
    Move,
    Resize,
}

impl Default for EditorTool {
    fn default() -> Self {
        Self::Select
    }
}

/// High-level editor mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorState {
    Idle,
    Editing,
    Testing,
    Saving,
}

impl Default for EditorState {
    fn default() -> Self {
        Self::Idle
    }
}

// ---------------------------------------------------------------------------
// Main application struct
// ---------------------------------------------------------------------------

/// Top-level editor application.
pub struct EditorApp {
    /// Current high-level state.
    pub state: EditorState,
    /// The level currently open for editing.
    pub current_level: Option<LevelDocument>,
    /// Active tool.
    pub tool: EditorTool,
    /// Currently selected entity IDs.
    pub selected_entities: Vec<u64>,
    /// Snap grid cell size in pixels.
    pub grid_size: f32,
    /// Whether to draw the grid overlay.
    pub show_grid: bool,
    /// Whether to draw physics debug shapes.
    pub show_physics: bool,
    /// Undo history (most recent at the back).
    pub undo_stack: Vec<EditorAction>,
    /// Redo history (most recent at the back).
    pub redo_stack: Vec<EditorAction>,
    /// Next unique ID for placed actors.
    next_actor_id: u64,
    /// Currently selected tile ID for the brush.
    pub selected_tile_id: u32,
    /// Current actor template name for placement.
    pub selected_actor_template: String,
    /// Zoom level (1.0 = 100 %).
    pub zoom: f32,
    /// Camera scroll offset.
    pub scroll_offset: (f32, f32),
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorApp {
    /// Create a new editor in idle state with default settings.
    pub fn new() -> Self {
        Self {
            state: EditorState::Idle,
            current_level: None,
            tool: EditorTool::Select,
            selected_entities: Vec::new(),
            grid_size: 64.0,
            show_grid: true,
            show_physics: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            next_actor_id: 1,
            selected_tile_id: 1,
            selected_actor_template: String::new(),
            zoom: 1.0,
            scroll_offset: (0.0, 0.0),
        }
    }

    /// Create a new empty level and open it for editing.
    pub fn new_level(&mut self, name: &str) {
        self.current_level = Some(LevelDocument::new(name));
        self.state = EditorState::Editing;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.selected_entities.clear();
        tracing::info!("Created new level '{name}'");
    }

    /// Apply an action to the current level and push it onto the undo stack.
    pub fn apply_action(&mut self, action: EditorAction) {
        if let Some(ref mut doc) = self.current_level {
            match &action.action_type {
                ActionType::PlaceTile { x, y, new, .. } => {
                    if let Some(layer) = doc.level_data.tile_layers.first_mut() {
                        layer.set(*x, *y, new.unwrap_or(0));
                    }
                }
                ActionType::PlaceActor {
                    id,
                    template,
                    x,
                    y,
                } => {
                    doc.level_data.actors.push(ActorInstance {
                        id: *id,
                        template: template.clone(),
                        x: *x,
                        y: *y,
                        properties: HashMap::new(),
                    });
                }
                ActionType::DeleteActor { id } => {
                    doc.level_data.actors.retain(|a| a.id != *id);
                }
                ActionType::MoveActor { id, new_pos, .. } => {
                    if let Some(actor) = doc.level_data.actors.iter_mut().find(|a| a.id == *id) {
                        actor.x = new_pos.0;
                        actor.y = new_pos.1;
                    }
                }
                ActionType::ModifyProperty {
                    entity_id,
                    property,
                    new_value,
                    ..
                } => {
                    if let Some(actor) = doc
                        .level_data
                        .actors
                        .iter_mut()
                        .find(|a| a.id == *entity_id)
                    {
                        actor.properties.insert(property.clone(), new_value.clone());
                    }
                }
            }
            doc.modified = true;
            self.undo_stack.push(action);
            self.redo_stack.clear();
        }
    }

    /// Undo the last action.
    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            if let Some(ref mut doc) = self.current_level {
                match &action.action_type {
                    ActionType::PlaceTile { x, y, old, .. } => {
                        if let Some(layer) = doc.level_data.tile_layers.first_mut() {
                            layer.set(*x, *y, old.unwrap_or(0));
                        }
                    }
                    ActionType::PlaceActor { id, .. } => {
                        doc.level_data.actors.retain(|a| a.id != *id);
                    }
                    ActionType::DeleteActor { id } => {
                        // Re-insert would require stored data; simplified here.
                        tracing::warn!("Undo DeleteActor({id}) — actor data not fully restored");
                    }
                    ActionType::MoveActor { id, old_pos, .. } => {
                        if let Some(actor) =
                            doc.level_data.actors.iter_mut().find(|a| a.id == *id)
                        {
                            actor.x = old_pos.0;
                            actor.y = old_pos.1;
                        }
                    }
                    ActionType::ModifyProperty {
                        entity_id,
                        property,
                        old_value,
                        ..
                    } => {
                        if let Some(actor) = doc
                            .level_data
                            .actors
                            .iter_mut()
                            .find(|a| a.id == *entity_id)
                        {
                            actor.properties.insert(property.clone(), old_value.clone());
                        }
                    }
                }
                doc.modified = true;
            }
            self.redo_stack.push(action);
        }
    }

    /// Redo the last undone action.
    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            self.apply_action(action);
        }
    }

    /// Allocate a fresh actor ID.
    pub fn next_actor_id(&mut self) -> u64 {
        let id = self.next_actor_id;
        self.next_actor_id += 1;
        id
    }

    /// Snap a world coordinate to the grid.
    pub fn snap_to_grid(&self, value: f32) -> f32 {
        if self.grid_size > 0.0 {
            (value / self.grid_size).round() * self.grid_size
        } else {
            value
        }
    }

    /// Convert a screen position to a world position accounting for zoom and scroll.
    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        (
            sx / self.zoom + self.scroll_offset.0,
            sy / self.zoom + self.scroll_offset.1,
        )
    }
}
