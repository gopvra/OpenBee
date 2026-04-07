//! Tile editing tools — brush, eraser, and flood-fill.

use crate::editor_app::{ActionType, EditorAction, EditorApp, TileLayer};

/// Paint a single tile at the given grid coordinates using the currently
/// selected tile ID.
pub fn paint_tile(app: &mut EditorApp, layer_index: usize, x: u32, y: u32, timestamp: f64) {
    if let Some(ref mut doc) = app.current_level {
        if let Some(layer) = doc.level_data.tile_layers.get_mut(layer_index) {
            let old = layer.get(x, y);
            let new_tile = app.selected_tile_id;
            if old == Some(new_tile) {
                return; // no-op
            }
            let action = EditorAction {
                action_type: ActionType::PlaceTile {
                    x,
                    y,
                    old,
                    new: Some(new_tile),
                },
                timestamp,
            };
            // Apply directly on layer (instead of going through apply_action
            // which also touches the layer) so we stay consistent.
            layer.set(x, y, new_tile);
            doc.modified = true;
            app.undo_stack.push(action);
            app.redo_stack.clear();
        }
    }
}

/// Erase a tile (set to 0) at the given grid coordinates.
pub fn erase_tile(app: &mut EditorApp, layer_index: usize, x: u32, y: u32, timestamp: f64) {
    if let Some(ref mut doc) = app.current_level {
        if let Some(layer) = doc.level_data.tile_layers.get_mut(layer_index) {
            let old = layer.get(x, y);
            if old == Some(0) {
                return;
            }
            let action = EditorAction {
                action_type: ActionType::PlaceTile {
                    x,
                    y,
                    old,
                    new: Some(0),
                },
                timestamp,
            };
            layer.set(x, y, 0);
            doc.modified = true;
            app.undo_stack.push(action);
            app.redo_stack.clear();
        }
    }
}

/// Flood-fill starting from `(sx, sy)` replacing tiles that match the target
/// with the currently selected tile ID. Uses a simple BFS.
pub fn flood_fill(
    layer: &mut TileLayer,
    sx: u32,
    sy: u32,
    fill_id: u32,
) -> Vec<(u32, u32, u32)> {
    let target = match layer.get(sx, sy) {
        Some(t) => t,
        None => return Vec::new(),
    };
    if target == fill_id {
        return Vec::new();
    }

    let mut changed: Vec<(u32, u32, u32)> = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((sx, sy));
    // Use a visited set to avoid re-processing.
    let mut visited = vec![false; (layer.width * layer.height) as usize];
    visited[(sy * layer.width + sx) as usize] = true;

    while let Some((cx, cy)) = queue.pop_front() {
        if layer.get(cx, cy) != Some(target) {
            continue;
        }
        changed.push((cx, cy, target));
        layer.set(cx, cy, fill_id);

        let neighbors: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in &neighbors {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && ny >= 0 {
                let (nx, ny) = (nx as u32, ny as u32);
                if nx < layer.width && ny < layer.height {
                    let idx = (ny * layer.width + nx) as usize;
                    if !visited[idx] {
                        visited[idx] = true;
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }

    changed
}

/// Draw the tile palette in a side panel so the user can pick a tile ID.
pub fn draw_tile_palette(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.heading("Tile Palette");
    ui.separator();

    // Simple grid of numbered buttons representing available tile IDs.
    let max_tile_id: u32 = 64; // placeholder upper bound
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for id in 1..=max_tile_id {
                let selected = app.selected_tile_id == id;
                let label = format!("{id}");
                if ui.selectable_label(selected, &label).clicked() {
                    app.selected_tile_id = id;
                }
            }
        });
    });
}
