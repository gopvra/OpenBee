//! In-game HUD: health bar, score, lives, ammo, minimap.

use glam::Vec2;
use openclaw_core::render::{Color, Rect, Renderer};

/// The in-game heads-up display showing player stats.
pub struct GameHud {
    /// Whether the HUD is visible.
    pub visible: bool,
    /// Screen dimensions for layout.
    screen_width: u32,
    screen_height: u32,
}

impl GameHud {
    /// Create a new HUD for the given screen dimensions.
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            visible: true,
            screen_width,
            screen_height,
        }
    }

    /// Draw the HUD overlay.
    pub fn draw(
        &self,
        renderer: &mut dyn Renderer,
        health: i32,
        max_health: i32,
        score: u64,
        lives: i32,
        pistol_ammo: i32,
        dynamite_ammo: i32,
        magic_ammo: i32,
    ) {
        if !self.visible {
            return;
        }

        let margin = 10.0;
        let bar_height = 16.0;
        let bar_width = 200.0;

        // Health bar background
        let health_bg = Rect::new(margin, margin, bar_width, bar_height);
        renderer.draw_rect_filled(health_bg, Color::rgb(64, 0, 0));

        // Health bar fill
        let health_fraction = if max_health > 0 {
            health as f32 / max_health as f32
        } else {
            0.0
        };
        let health_fill = Rect::new(margin, margin, bar_width * health_fraction, bar_height);
        renderer.draw_rect_filled(health_fill, Color::rgb(255, 0, 0));

        // Health bar border
        renderer.draw_rect_outline(health_bg, Color::WHITE);

        // Health text
        renderer.draw_text(
            &format!("{}/{}", health, max_health),
            Vec2::new(margin + bar_width + 8.0, margin),
            14.0,
            Color::WHITE,
        );

        // Score
        renderer.draw_text(
            &format!("SCORE: {}", score),
            Vec2::new(self.screen_width as f32 / 2.0 - 60.0, margin),
            16.0,
            Color::rgb(255, 255, 0),
        );

        // Lives
        renderer.draw_text(
            &format!("LIVES: {}", lives),
            Vec2::new(self.screen_width as f32 - 120.0, margin),
            14.0,
            Color::WHITE,
        );

        // Ammo display
        let ammo_y = self.screen_height as f32 - 30.0;
        renderer.draw_text(
            &format!("P:{}", pistol_ammo),
            Vec2::new(margin, ammo_y),
            12.0,
            Color::rgb(200, 200, 200),
        );
        renderer.draw_text(
            &format!("D:{}", dynamite_ammo),
            Vec2::new(margin + 60.0, ammo_y),
            12.0,
            Color::rgb(255, 128, 0),
        );
        renderer.draw_text(
            &format!("M:{}", magic_ammo),
            Vec2::new(margin + 120.0, ammo_y),
            12.0,
            Color::rgb(128, 128, 255),
        );
    }

    /// Draw a minimap in the corner of the screen.
    pub fn draw_minimap(
        &self,
        renderer: &mut dyn Renderer,
        player_x: f32,
        player_y: f32,
        level_width: f32,
        level_height: f32,
    ) {
        if !self.visible {
            return;
        }

        let minimap_size = 100.0;
        let minimap_x = self.screen_width as f32 - minimap_size - 10.0;
        let minimap_y = self.screen_height as f32 - minimap_size - 40.0;

        // Minimap background
        let bg = Rect::new(minimap_x, minimap_y, minimap_size, minimap_size);
        renderer.draw_rect_filled(bg, Color::new(0, 0, 0, 128));
        renderer.draw_rect_outline(bg, Color::rgb(128, 128, 128));

        // Player dot
        if level_width > 0.0 && level_height > 0.0 {
            let px = minimap_x + (player_x / level_width) * minimap_size;
            let py = minimap_y + (player_y / level_height) * minimap_size;
            let dot = Rect::new(px - 2.0, py - 2.0, 4.0, 4.0);
            renderer.draw_rect_filled(dot, Color::rgb(0, 255, 0));
        }
    }
}
