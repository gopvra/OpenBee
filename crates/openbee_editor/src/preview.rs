//! Live level preview renderer — draws the tile map and actor icons into an
//! egui painter region.

use egui::{Color32, Pos2, Rect as EguiRect, Stroke, StrokeKind, Ui, Vec2 as EguiVec2};

use crate::editor_app::EditorApp;

/// Draw the level preview in the main viewport area.
pub fn draw_preview(ui: &mut Ui, app: &EditorApp) {
    let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
    let canvas_rect = response.rect;
    let origin = canvas_rect.min;

    let zoom = app.zoom;
    let (scroll_x, scroll_y) = app.scroll_offset;

    // Draw grid
    if app.show_grid {
        draw_grid(
            &painter,
            canvas_rect,
            app.grid_size,
            zoom,
            scroll_x,
            scroll_y,
        );
    }

    let Some(ref doc) = app.current_level else {
        // Nothing to draw.
        painter.text(
            canvas_rect.center(),
            egui::Align2::CENTER_CENTER,
            "No level loaded",
            egui::FontId::proportional(24.0),
            Color32::GRAY,
        );
        return;
    };

    let data = &doc.level_data;

    // Draw tile layers.
    for layer in &data.tile_layers {
        if !layer.visible {
            continue;
        }
        for ty in 0..layer.height {
            for tx in 0..layer.width {
                let tile_id = layer.tiles[(ty * layer.width + tx) as usize];
                if tile_id == 0 {
                    continue;
                }
                let px = (tx as f32 * data.tile_size as f32 - scroll_x) * zoom + origin.x;
                let py = (ty as f32 * data.tile_size as f32 - scroll_y) * zoom + origin.y;
                let size = data.tile_size as f32 * zoom;

                // Skip tiles outside the visible canvas.
                if px + size < canvas_rect.min.x
                    || py + size < canvas_rect.min.y
                    || px > canvas_rect.max.x
                    || py > canvas_rect.max.y
                {
                    continue;
                }

                // Pick a colour based on tile ID for a visual distinction.
                let color = tile_id_to_color(tile_id);
                let rect = EguiRect::from_min_size(Pos2::new(px, py), EguiVec2::new(size, size));
                painter.rect_filled(rect, 0.0, color);
                painter.rect_stroke(
                    rect,
                    0.0,
                    Stroke::new(1.0, Color32::DARK_GRAY),
                    StrokeKind::Outside,
                );
            }
        }
    }

    // Draw actors.
    let actor_half = 12.0 * zoom;
    for actor in &data.actors {
        let px = (actor.x - scroll_x) * zoom + origin.x;
        let py = (actor.y - scroll_y) * zoom + origin.y;

        let rect = EguiRect::from_center_size(
            Pos2::new(px, py),
            EguiVec2::new(actor_half * 2.0, actor_half * 2.0),
        );

        let is_selected = app.selected_entities.contains(&actor.id);
        let fill = if is_selected {
            Color32::from_rgb(255, 200, 50)
        } else {
            Color32::from_rgb(50, 180, 255)
        };

        painter.rect_filled(rect, 4.0, fill);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::WHITE),
            StrokeKind::Outside,
        );

        // Label
        painter.text(
            Pos2::new(px, py - actor_half - 4.0),
            egui::Align2::CENTER_BOTTOM,
            &actor.template,
            egui::FontId::proportional(10.0 * zoom.max(0.5)),
            Color32::WHITE,
        );
    }

    // Draw physics debug outlines (placeholder boxes).
    if app.show_physics {
        for actor in &data.actors {
            let px = (actor.x - scroll_x) * zoom + origin.x;
            let py = (actor.y - scroll_y) * zoom + origin.y;
            let half = 16.0 * zoom;
            let rect = EguiRect::from_center_size(
                Pos2::new(px, py),
                EguiVec2::new(half * 2.0, half * 2.0),
            );
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, Color32::GREEN),
                StrokeKind::Outside,
            );
        }
    }
}

/// Draw an infinite grid overlay.
fn draw_grid(
    painter: &egui::Painter,
    canvas: EguiRect,
    grid_size: f32,
    zoom: f32,
    scroll_x: f32,
    scroll_y: f32,
) {
    let step = grid_size * zoom;
    if step < 4.0 {
        return; // too dense to draw
    }

    let color = Color32::from_rgba_premultiplied(80, 80, 80, 40);
    let stroke = Stroke::new(1.0, color);

    let offset_x = (-scroll_x * zoom) % step;
    let offset_y = (-scroll_y * zoom) % step;

    // Vertical lines
    let mut x = canvas.min.x + offset_x;
    while x <= canvas.max.x {
        painter.line_segment(
            [Pos2::new(x, canvas.min.y), Pos2::new(x, canvas.max.y)],
            stroke,
        );
        x += step;
    }

    // Horizontal lines
    let mut y = canvas.min.y + offset_y;
    while y <= canvas.max.y {
        painter.line_segment(
            [Pos2::new(canvas.min.x, y), Pos2::new(canvas.max.x, y)],
            stroke,
        );
        y += step;
    }
}

/// Map a tile ID to a deterministic pastel colour.
fn tile_id_to_color(id: u32) -> Color32 {
    // Simple hash-ish mapping.
    let r = ((id.wrapping_mul(47)) % 128 + 100) as u8;
    let g = ((id.wrapping_mul(113)) % 128 + 100) as u8;
    let b = ((id.wrapping_mul(199)) % 128 + 100) as u8;
    Color32::from_rgb(r, g, b)
}
