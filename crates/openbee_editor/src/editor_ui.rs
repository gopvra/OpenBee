//! egui-based main UI layout — menu bar, toolbar, status bar.

use egui::Ui;

use crate::editor_app::{EditorApp, EditorState, EditorTool};

/// Draw the top menu bar (File, Edit, View, Help).
pub fn draw_menu_bar(ui: &mut Ui, app: &mut EditorApp) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New Level").clicked() {
                app.new_level("Untitled");
                ui.close_menu();
            }
            if ui.button("Open...").clicked() {
                tracing::info!("Open level dialog requested");
                ui.close_menu();
            }
            if ui.button("Save").clicked() {
                app.state = EditorState::Saving;
                ui.close_menu();
            }
            if ui.button("Save As...").clicked() {
                app.state = EditorState::Saving;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Exit").clicked() {
                tracing::info!("Exit requested");
                ui.close_menu();
            }
        });

        ui.menu_button("Edit", |ui| {
            let can_undo = !app.undo_stack.is_empty();
            let can_redo = !app.redo_stack.is_empty();
            if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
                app.undo();
                ui.close_menu();
            }
            if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
                app.redo();
                ui.close_menu();
            }
        });

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut app.show_grid, "Show Grid");
            ui.checkbox(&mut app.show_physics, "Show Physics");
            ui.separator();
            if ui.button("Zoom In").clicked() {
                app.zoom = (app.zoom * 1.25).min(8.0);
                ui.close_menu();
            }
            if ui.button("Zoom Out").clicked() {
                app.zoom = (app.zoom / 1.25).max(0.125);
                ui.close_menu();
            }
            if ui.button("Reset Zoom").clicked() {
                app.zoom = 1.0;
                ui.close_menu();
            }
        });

        ui.menu_button("Help", |ui| {
            if ui.button("About").clicked() {
                tracing::info!("About dialog");
                ui.close_menu();
            }
        });
    });
}

/// Draw the horizontal toolbar with tool selection buttons.
pub fn draw_toolbar(ui: &mut Ui, app: &mut EditorApp) {
    ui.horizontal(|ui| {
        let tools = [
            (EditorTool::Select, "Select"),
            (EditorTool::TileBrush, "Brush"),
            (EditorTool::TileEraser, "Eraser"),
            (EditorTool::ActorPlace, "Actor"),
            (EditorTool::TriggerZone, "Trigger"),
            (EditorTool::Move, "Move"),
            (EditorTool::Resize, "Resize"),
        ];
        for (tool, label) in &tools {
            let selected = app.tool == *tool;
            if ui.selectable_label(selected, *label).clicked() {
                app.tool = *tool;
            }
        }

        ui.separator();

        // Grid size slider
        ui.label("Grid:");
        ui.add(egui::Slider::new(&mut app.grid_size, 8.0..=128.0).suffix("px"));

        ui.separator();

        // Test button
        if app.state == EditorState::Testing {
            if ui.button("Stop Test").clicked() {
                app.state = EditorState::Editing;
            }
        } else if app.current_level.is_some() && ui.button("Test Level").clicked() {
            app.state = EditorState::Testing;
        }
    });
}

/// Draw the bottom status bar.
pub fn draw_status_bar(ui: &mut Ui, app: &EditorApp) {
    ui.horizontal(|ui| {
        // State
        let state_str = match app.state {
            EditorState::Idle => "Idle",
            EditorState::Editing => "Editing",
            EditorState::Testing => "Testing",
            EditorState::Saving => "Saving...",
        };
        ui.label(format!("State: {state_str}"));

        ui.separator();

        // Level name
        if let Some(ref doc) = app.current_level {
            let modified = if doc.modified { " *" } else { "" };
            ui.label(format!("Level: {}{modified}", doc.name));
        } else {
            ui.label("No level open");
        }

        ui.separator();

        // Selection info
        let sel_count = app.selected_entities.len();
        if sel_count > 0 {
            ui.label(format!("Selected: {sel_count} entities"));
        }

        ui.separator();

        // Tool
        ui.label(format!("Tool: {:?}", app.tool));

        ui.separator();

        // Zoom
        ui.label(format!("Zoom: {:.0}%", app.zoom * 100.0));
    });
}
