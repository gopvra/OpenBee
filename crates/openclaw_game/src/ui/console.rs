//! Debug console for in-game commands.

use glam::Vec2;
use openclaw_core::render::{Color, Rect, Renderer};

/// Maximum number of output lines to keep in the console log.
const MAX_LOG_LINES: usize = 100;

/// An in-game debug console for developer commands.
pub struct DebugConsole {
    /// Whether the console is currently visible/active.
    pub visible: bool,
    /// Current command input buffer.
    input_buffer: String,
    /// Console output log.
    output_log: Vec<String>,
    /// Command history for up/down navigation.
    history: Vec<String>,
    /// Current position in command history.
    history_index: usize,
    /// Scroll offset for the output log.
    scroll_offset: usize,
}

impl DebugConsole {
    /// Create a new hidden debug console.
    pub fn new() -> Self {
        Self {
            visible: false,
            input_buffer: String::new(),
            output_log: Vec::new(),
            history: Vec::new(),
            history_index: 0,
            scroll_offset: 0,
        }
    }

    /// Toggle console visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Add a character to the input buffer.
    pub fn input_char(&mut self, c: char) {
        if self.visible {
            self.input_buffer.push(c);
        }
    }

    /// Delete the last character from the input buffer.
    pub fn backspace(&mut self) {
        if self.visible {
            self.input_buffer.pop();
        }
    }

    /// Navigate to the previous command in history.
    pub fn history_up(&mut self) {
        if !self.history.is_empty() && self.history_index > 0 {
            self.history_index -= 1;
            self.input_buffer = self.history[self.history_index].clone();
        }
    }

    /// Navigate to the next command in history.
    pub fn history_down(&mut self) {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            if self.history_index < self.history.len() {
                self.input_buffer = self.history[self.history_index].clone();
            } else {
                self.input_buffer.clear();
            }
        }
    }

    /// Scroll the output log up.
    pub fn scroll_up(&mut self) {
        if self.scroll_offset < self.output_log.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    /// Scroll the output log down.
    pub fn scroll_down(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Submit the current input buffer as a command.
    pub fn submit(&mut self) -> Option<String> {
        if !self.visible || self.input_buffer.is_empty() {
            return None;
        }
        let command = self.input_buffer.clone();
        self.history.push(command.clone());
        self.history_index = self.history.len();
        self.input_buffer.clear();
        self.log(&format!("> {}", command));
        Some(command)
    }

    /// Add a line to the output log.
    pub fn log(&mut self, message: &str) {
        self.output_log.push(message.to_string());
        if self.output_log.len() > MAX_LOG_LINES {
            self.output_log.remove(0);
        }
    }

    /// Execute a console command and return the result message.
    pub fn execute_command(&mut self, command: &str) -> String {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }

        let result = match parts[0] {
            "help" => {
                "Available commands: help, god, noclip, give, teleport, level, spawn, kill, fps, quit"
                    .to_string()
            }
            "god" => "God mode toggled".to_string(),
            "noclip" => "Noclip toggled".to_string(),
            "give" => {
                if parts.len() < 3 {
                    "Usage: give <item> <amount>".to_string()
                } else {
                    format!("Gave {} {}", parts[2], parts[1])
                }
            }
            "teleport" | "tp" => {
                if parts.len() < 3 {
                    "Usage: teleport <x> <y>".to_string()
                } else {
                    format!("Teleported to ({}, {})", parts[1], parts[2])
                }
            }
            "level" => {
                if parts.len() < 2 {
                    "Usage: level <number>".to_string()
                } else {
                    format!("Loading level {}", parts[1])
                }
            }
            "spawn" => {
                if parts.len() < 2 {
                    "Usage: spawn <actor_name>".to_string()
                } else {
                    format!("Spawned {}", parts[1])
                }
            }
            "kill" => "Killed all enemies".to_string(),
            "fps" => "FPS display toggled".to_string(),
            "quit" | "exit" => "Quitting...".to_string(),
            _ => format!("Unknown command: {}", parts[0]),
        };

        self.log(&result);
        result
    }

    /// Draw the console overlay.
    pub fn draw(&self, renderer: &mut dyn Renderer) {
        if !self.visible {
            return;
        }

        let (sw, _sh) = renderer.screen_size();
        let console_height = 300.0;

        // Background
        renderer.draw_rect_filled(
            Rect::new(0.0, 0.0, sw as f32, console_height),
            Color::new(0, 0, 0, 220),
        );

        // Output log
        let visible_lines = 15;
        let start = self
            .output_log
            .len()
            .saturating_sub(visible_lines + self.scroll_offset);
        let end = self.output_log.len().saturating_sub(self.scroll_offset);

        for (i, line) in self.output_log[start..end].iter().enumerate() {
            renderer.draw_text(
                line,
                Vec2::new(8.0, 8.0 + i as f32 * 16.0),
                12.0,
                Color::rgb(200, 200, 200),
            );
        }

        // Input line
        let input_y = console_height - 24.0;
        renderer.draw_rect_filled(
            Rect::new(0.0, input_y, sw as f32, 24.0),
            Color::new(40, 40, 40, 255),
        );
        renderer.draw_text(
            &format!("> {}_", self.input_buffer),
            Vec2::new(8.0, input_y + 4.0),
            12.0,
            Color::rgb(0, 255, 0),
        );
    }
}

impl Default for DebugConsole {
    fn default() -> Self {
        Self::new()
    }
}
