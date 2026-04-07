//! Main menu, pause menu, and options menu.

use glam::Vec2;
use openclaw_core::render::{Color, Rect, Renderer};

/// Result of a menu selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSelection {
    None,
    NewGame,
    LoadGame,
    SaveGame,
    Options,
    Resume,
    MainMenu,
    Quit,
}

/// Common menu behavior.
struct MenuBase {
    items: Vec<String>,
    selected_index: usize,
}

impl MenuBase {
    fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected_index: 0,
        }
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.items.len().saturating_sub(1);
        }
    }

    fn move_down(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    fn draw(&self, renderer: &mut dyn Renderer, start_y: f32, title: &str) {
        let (sw, _sh) = renderer.screen_size();
        let center_x = sw as f32 / 2.0;

        // Title
        renderer.draw_text(title, Vec2::new(center_x - 80.0, start_y), 24.0, Color::rgb(255, 215, 0));

        // Menu items
        for (i, item) in self.items.iter().enumerate() {
            let y = start_y + 60.0 + i as f32 * 30.0;
            let color = if i == self.selected_index {
                Color::rgb(255, 255, 0)
            } else {
                Color::rgb(200, 200, 200)
            };
            renderer.draw_text(item, Vec2::new(center_x - 60.0, y), 18.0, color);

            // Selection indicator
            if i == self.selected_index {
                renderer.draw_text(">", Vec2::new(center_x - 80.0, y), 18.0, Color::rgb(255, 255, 0));
            }
        }
    }
}

/// The title screen main menu.
pub struct MainMenu {
    menu: MenuBase,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            menu: MenuBase::new(vec![
                "New Game".to_string(),
                "Load Game".to_string(),
                "Options".to_string(),
                "Quit".to_string(),
            ]),
        }
    }

    pub fn move_up(&mut self) {
        self.menu.move_up();
    }

    pub fn move_down(&mut self) {
        self.menu.move_down();
    }

    pub fn confirm(&self) -> MenuSelection {
        match self.menu.selected_index {
            0 => MenuSelection::NewGame,
            1 => MenuSelection::LoadGame,
            2 => MenuSelection::Options,
            3 => MenuSelection::Quit,
            _ => MenuSelection::None,
        }
    }

    pub fn draw(&self, renderer: &mut dyn Renderer) {
        let (_, _sh) = renderer.screen_size();
        // Dark overlay
        let (sw, sh) = renderer.screen_size();
        renderer.draw_rect_filled(
            Rect::new(0.0, 0.0, sw as f32, sh as f32),
            Color::new(0, 0, 0, 200),
        );
        self.menu.draw(renderer, sh as f32 * 0.25, "CAPTAIN CLAW");
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// The in-game pause menu.
pub struct PauseMenu {
    menu: MenuBase,
}

impl PauseMenu {
    pub fn new() -> Self {
        Self {
            menu: MenuBase::new(vec![
                "Resume".to_string(),
                "Save Game".to_string(),
                "Options".to_string(),
                "Main Menu".to_string(),
                "Quit".to_string(),
            ]),
        }
    }

    pub fn move_up(&mut self) {
        self.menu.move_up();
    }

    pub fn move_down(&mut self) {
        self.menu.move_down();
    }

    pub fn confirm(&self) -> MenuSelection {
        match self.menu.selected_index {
            0 => MenuSelection::Resume,
            1 => MenuSelection::SaveGame,
            2 => MenuSelection::Options,
            3 => MenuSelection::MainMenu,
            4 => MenuSelection::Quit,
            _ => MenuSelection::None,
        }
    }

    pub fn draw(&self, renderer: &mut dyn Renderer) {
        let (sw, sh) = renderer.screen_size();
        renderer.draw_rect_filled(
            Rect::new(0.0, 0.0, sw as f32, sh as f32),
            Color::new(0, 0, 0, 180),
        );
        self.menu.draw(renderer, sh as f32 * 0.3, "PAUSED");
    }
}

impl Default for PauseMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// Options menu for adjusting settings.
pub struct OptionsMenu {
    menu: MenuBase,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub fullscreen: bool,
}

impl OptionsMenu {
    pub fn new() -> Self {
        Self {
            menu: MenuBase::new(vec![
                "Master Volume".to_string(),
                "Music Volume".to_string(),
                "SFX Volume".to_string(),
                "Fullscreen".to_string(),
                "Back".to_string(),
            ]),
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 0.8,
            fullscreen: false,
        }
    }

    pub fn move_up(&mut self) {
        self.menu.move_up();
    }

    pub fn move_down(&mut self) {
        self.menu.move_down();
    }

    /// Adjust the currently selected option left/right.
    pub fn adjust(&mut self, delta: f32) {
        match self.menu.selected_index {
            0 => self.master_volume = (self.master_volume + delta).clamp(0.0, 1.0),
            1 => self.music_volume = (self.music_volume + delta).clamp(0.0, 1.0),
            2 => self.sfx_volume = (self.sfx_volume + delta).clamp(0.0, 1.0),
            3 => self.fullscreen = !self.fullscreen,
            _ => {}
        }
    }

    /// Returns true if "Back" is selected and confirmed.
    pub fn confirm(&self) -> bool {
        self.menu.selected_index == 4
    }

    pub fn draw(&self, renderer: &mut dyn Renderer) {
        let (sw, sh) = renderer.screen_size();
        renderer.draw_rect_filled(
            Rect::new(0.0, 0.0, sw as f32, sh as f32),
            Color::new(0, 0, 0, 200),
        );

        let start_y = sh as f32 * 0.25;
        let center_x = sw as f32 / 2.0;

        renderer.draw_text("OPTIONS", Vec2::new(center_x - 50.0, start_y), 24.0, Color::rgb(255, 215, 0));

        let values = [
            format!("{:.0}%", self.master_volume * 100.0),
            format!("{:.0}%", self.music_volume * 100.0),
            format!("{:.0}%", self.sfx_volume * 100.0),
            if self.fullscreen { "ON".to_string() } else { "OFF".to_string() },
            String::new(),
        ];

        for (i, item) in self.menu.items.iter().enumerate() {
            let y = start_y + 60.0 + i as f32 * 30.0;
            let color = if i == self.menu.selected_index {
                Color::rgb(255, 255, 0)
            } else {
                Color::rgb(200, 200, 200)
            };
            renderer.draw_text(item, Vec2::new(center_x - 100.0, y), 18.0, color);
            if !values[i].is_empty() {
                renderer.draw_text(&values[i], Vec2::new(center_x + 60.0, y), 18.0, color);
            }
        }
    }
}

impl Default for OptionsMenu {
    fn default() -> Self {
        Self::new()
    }
}
