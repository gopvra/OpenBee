//! Property inspector panel — displays and edits entity component data.

use crate::editor_app::{ActionType, EditorAction, EditorApp};

/// Draw the property inspector for the currently selected entities.
pub fn draw_property_panel(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.heading("Properties");
    ui.separator();

    if app.selected_entities.is_empty() {
        ui.label("No entity selected.");
        return;
    }

    // Show properties for the first selected entity.
    let entity_id = app.selected_entities[0];

    // Find the actor in the level data.
    let actor = app
        .current_level
        .as_ref()
        .and_then(|doc| doc.level_data.actors.iter().find(|a| a.id == entity_id))
        .cloned();

    let Some(actor) = actor else {
        ui.label(format!("Entity {entity_id} not found."));
        return;
    };

    ui.label(format!("ID: {}", actor.id));
    ui.label(format!("Template: {}", actor.template));
    ui.separator();

    // Editable position
    let mut x = actor.x;
    let mut y = actor.y;
    let mut pos_changed = false;

    ui.horizontal(|ui| {
        ui.label("X:");
        if ui.add(egui::DragValue::new(&mut x).speed(1.0)).changed() {
            pos_changed = true;
        }
    });
    ui.horizontal(|ui| {
        ui.label("Y:");
        if ui.add(egui::DragValue::new(&mut y).speed(1.0)).changed() {
            pos_changed = true;
        }
    });

    if pos_changed {
        if let Some(ref mut doc) = app.current_level {
            if let Some(a) = doc.level_data.actors.iter_mut().find(|a| a.id == entity_id) {
                let old_pos = (a.x, a.y);
                a.x = x;
                a.y = y;
                doc.modified = true;
                let action = EditorAction {
                    action_type: ActionType::MoveActor {
                        id: entity_id,
                        old_pos,
                        new_pos: (x, y),
                    },
                    timestamp: 0.0,
                };
                app.undo_stack.push(action);
                app.redo_stack.clear();
            }
        }
    }

    ui.separator();
    ui.label("Custom Properties:");

    // Display each custom property as an editable text field.
    let props: Vec<(String, String)> = actor.properties.into_iter().collect();
    for (key, value) in props {
        let mut val = value.clone();
        ui.horizontal(|ui| {
            ui.label(&key);
            if ui.text_edit_singleline(&mut val).changed() {
                if let Some(ref mut doc) = app.current_level {
                    if let Some(a) = doc.level_data.actors.iter_mut().find(|a| a.id == entity_id) {
                        a.properties.insert(key.clone(), val.clone());
                        doc.modified = true;
                        let action = EditorAction {
                            action_type: ActionType::ModifyProperty {
                                entity_id,
                                property: key.clone(),
                                old_value: value.clone(),
                                new_value: val,
                            },
                            timestamp: 0.0,
                        };
                        app.undo_stack.push(action);
                        app.redo_stack.clear();
                    }
                }
            }
        });
    }

    // Button to add a new property.
    if ui.button("+ Add Property").clicked() {
        if let Some(ref mut doc) = app.current_level {
            if let Some(a) = doc.level_data.actors.iter_mut().find(|a| a.id == entity_id) {
                let key = format!("prop_{}", a.properties.len());
                a.properties.insert(key, String::new());
                doc.modified = true;
            }
        }
    }
}
