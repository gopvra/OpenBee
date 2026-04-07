//! Actor editing — placement, selection, drag-move.

use crate::editor_app::{ActionType, ActorInstance, EditorAction, EditorApp};
use std::collections::HashMap;

/// Place a new actor at the given world position using the currently selected
/// template. Returns the new actor ID.
pub fn place_actor(app: &mut EditorApp, wx: f32, wy: f32, timestamp: f64) -> Option<u64> {
    if app.selected_actor_template.is_empty() {
        tracing::warn!("No actor template selected");
        return None;
    }

    let id = app.next_actor_id();
    let template = app.selected_actor_template.clone();

    let action = EditorAction {
        action_type: ActionType::PlaceActor {
            id,
            template: template.clone(),
            x: wx,
            y: wy,
        },
        timestamp,
    };

    if let Some(ref mut doc) = app.current_level {
        doc.level_data.actors.push(ActorInstance {
            id,
            template,
            x: wx,
            y: wy,
            properties: HashMap::new(),
        });
        doc.modified = true;
    }
    app.undo_stack.push(action);
    app.redo_stack.clear();

    tracing::debug!("Placed actor {id} at ({wx}, {wy})");
    Some(id)
}

/// Delete the actor with the given ID.
pub fn delete_actor(app: &mut EditorApp, actor_id: u64, timestamp: f64) {
    let action = EditorAction {
        action_type: ActionType::DeleteActor { id: actor_id },
        timestamp,
    };
    if let Some(ref mut doc) = app.current_level {
        doc.level_data.actors.retain(|a| a.id != actor_id);
        doc.modified = true;
    }
    app.selected_entities.retain(|&id| id != actor_id);
    app.undo_stack.push(action);
    app.redo_stack.clear();
}

/// Begin moving a selected actor — records the starting position.
pub struct DragState {
    pub actor_id: u64,
    pub start_x: f32,
    pub start_y: f32,
}

/// Complete a drag-move, recording the action for undo.
pub fn finish_move(app: &mut EditorApp, drag: &DragState, new_x: f32, new_y: f32, timestamp: f64) {
    if (drag.start_x - new_x).abs() < 0.001 && (drag.start_y - new_y).abs() < 0.001 {
        return; // no movement
    }

    let action = EditorAction {
        action_type: ActionType::MoveActor {
            id: drag.actor_id,
            old_pos: (drag.start_x, drag.start_y),
            new_pos: (new_x, new_y),
        },
        timestamp,
    };

    if let Some(ref mut doc) = app.current_level {
        if let Some(actor) = doc
            .level_data
            .actors
            .iter_mut()
            .find(|a| a.id == drag.actor_id)
        {
            actor.x = new_x;
            actor.y = new_y;
        }
        doc.modified = true;
    }
    app.undo_stack.push(action);
    app.redo_stack.clear();
}

/// Find the topmost actor at a world position (simple rectangular hit-test).
pub fn pick_actor(app: &EditorApp, wx: f32, wy: f32, hit_size: f32) -> Option<u64> {
    let half = hit_size / 2.0;
    if let Some(ref doc) = app.current_level {
        // Iterate in reverse so the most-recently-placed actor wins.
        for actor in doc.level_data.actors.iter().rev() {
            if wx >= actor.x - half
                && wx <= actor.x + half
                && wy >= actor.y - half
                && wy <= actor.y + half
            {
                return Some(actor.id);
            }
        }
    }
    None
}

/// Draw the actor template palette in a side panel.
pub fn draw_actor_palette(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.heading("Actor Templates");
    ui.separator();

    let templates = [
        "Officer",
        "Soldier",
        "RatBomber",
        "Treasure",
        "HealthPotion",
        "Checkpoint",
        "EndLevelTrigger",
        "MovingPlatform",
        "SpringBoard",
        "Crate",
        "Rope",
        "Cannon",
    ];

    egui::ScrollArea::vertical().show(ui, |ui| {
        for tmpl in &templates {
            let selected = app.selected_actor_template == *tmpl;
            if ui.selectable_label(selected, *tmpl).clicked() {
                app.selected_actor_template = tmpl.to_string();
            }
        }
    });
}
