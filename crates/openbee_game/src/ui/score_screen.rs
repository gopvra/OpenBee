//! Score screen displayed at the end of each level.

use glam::Vec2;
use openbee_core::render::{Color, Rect, Renderer};

/// End-of-level statistics for display.
#[derive(Debug, Clone)]
pub struct LevelStats {
    pub level_name: String,
    pub level_number: u32,
    pub score: u64,
    pub time_seconds: f32,
    pub enemies_killed: u32,
    pub enemies_total: u32,
    pub treasures_found: u32,
    pub treasures_total: u32,
    pub secrets_found: u32,
    pub secrets_total: u32,
}

impl Default for LevelStats {
    fn default() -> Self {
        Self {
            level_name: String::new(),
            level_number: 0,
            score: 0,
            time_seconds: 0.0,
            enemies_killed: 0,
            enemies_total: 0,
            treasures_found: 0,
            treasures_total: 0,
            secrets_found: 0,
            secrets_total: 0,
        }
    }
}

/// The score screen shown between levels.
pub struct ScoreScreen {
    pub stats: LevelStats,
    /// Animation timer for the score tally.
    tally_timer: f32,
    /// Whether the tally animation is complete.
    tally_complete: bool,
}

impl ScoreScreen {
    /// Create a new score screen with the given stats.
    pub fn new(stats: LevelStats) -> Self {
        Self {
            stats,
            tally_timer: 0.0,
            tally_complete: false,
        }
    }

    /// Update the tally animation.
    pub fn update(&mut self, dt: f32) {
        if !self.tally_complete {
            self.tally_timer += dt;
            if self.tally_timer >= 3.0 {
                self.tally_complete = true;
            }
        }
    }

    /// Whether the score screen is done and the player can continue.
    pub fn is_complete(&self) -> bool {
        self.tally_complete
    }

    /// Skip the tally animation.
    pub fn skip(&mut self) {
        self.tally_complete = true;
        self.tally_timer = 3.0;
    }

    /// Format seconds as MM:SS.
    fn format_time(seconds: f32) -> String {
        let mins = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Draw the score screen.
    pub fn draw(&self, renderer: &mut dyn Renderer) {
        let (sw, sh) = renderer.screen_size();

        // Dark background
        renderer.draw_rect_filled(
            Rect::new(0.0, 0.0, sw as f32, sh as f32),
            Color::new(0, 0, 20, 240),
        );

        let center_x = sw as f32 / 2.0;
        let start_y = 80.0;

        // Level name
        renderer.draw_text(
            &format!(
                "Level {} - {}",
                self.stats.level_number, self.stats.level_name
            ),
            Vec2::new(center_x - 120.0, start_y),
            24.0,
            Color::rgb(255, 215, 0),
        );

        // Tally progress (animate reveals)
        let progress = (self.tally_timer / 3.0).min(1.0);

        let lines: Vec<(String, String)> = vec![
            ("SCORE".to_string(), format!("{}", self.stats.score)),
            (
                "TIME".to_string(),
                Self::format_time(self.stats.time_seconds),
            ),
            (
                "ENEMIES".to_string(),
                format!("{}/{}", self.stats.enemies_killed, self.stats.enemies_total),
            ),
            (
                "TREASURES".to_string(),
                format!(
                    "{}/{}",
                    self.stats.treasures_found, self.stats.treasures_total
                ),
            ),
            (
                "SECRETS".to_string(),
                format!("{}/{}", self.stats.secrets_found, self.stats.secrets_total),
            ),
        ];

        let total_lines = lines.len();
        let lines_to_show = ((progress * total_lines as f32).ceil() as usize).min(total_lines);

        for (i, (label, value)) in lines[..lines_to_show].iter().enumerate() {
            let y = start_y + 60.0 + i as f32 * 35.0;
            renderer.draw_text(label, Vec2::new(center_x - 120.0, y), 18.0, Color::WHITE);
            renderer.draw_text(
                value,
                Vec2::new(center_x + 60.0, y),
                18.0,
                Color::rgb(255, 255, 0),
            );
        }

        // Continue prompt
        if self.tally_complete {
            renderer.draw_text(
                "Press any key to continue",
                Vec2::new(center_x - 100.0, sh as f32 - 60.0),
                14.0,
                Color::rgb(200, 200, 200),
            );
        }
    }
}
