//! Speedrun timer with splits, personal bests, and display formatting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum allowed size for speedrun save files (1 MB).
const MAX_SAVE_SIZE: u64 = 1024 * 1024;

/// A full-featured speedrun timer supporting splits, personal bests, and delta tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedrunTimer {
    /// Total elapsed time in milliseconds.
    pub total_time_ms: u64,
    /// Whether the timer is currently running.
    pub running: bool,
    /// Ordered list of splits for the current run.
    pub splits: Vec<Split>,
    /// Index of the current (next unfinished) split.
    pub current_split: usize,
    /// Best time for each split by name across all runs.
    pub best_splits: HashMap<String, u64>,
    /// Overall personal best total time, if one exists.
    pub personal_best: Option<u64>,
    /// Whether the timer overlay should be drawn.
    pub display_enabled: bool,
    /// Screen position (x, y) for the timer overlay.
    pub display_position: (f32, f32),
    /// Automatically start the timer when a level begins.
    pub auto_start_on_level: bool,
    /// Automatically record a split when a level is completed.
    pub auto_split_on_level_complete: bool,
}

/// A single split (segment) of a speedrun.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Split {
    /// Human-readable name of this split (e.g. level name).
    pub name: String,
    /// Cumulative time at which this split was reached, or `None` if not yet reached.
    pub time_ms: Option<u64>,
    /// Personal best cumulative time for this split.
    pub best_time_ms: Option<u64>,
    /// Difference from the best time (positive = behind, negative = ahead).
    pub delta_ms: Option<i64>,
}

impl SpeedrunTimer {
    /// Create a new timer in the stopped/reset state.
    pub fn new() -> Self {
        Self {
            total_time_ms: 0,
            running: false,
            splits: Vec::new(),
            current_split: 0,
            best_splits: HashMap::new(),
            personal_best: None,
            display_enabled: true,
            display_position: (10.0, 10.0),
            auto_start_on_level: false,
            auto_split_on_level_complete: false,
        }
    }

    /// Start (or resume) the timer.
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Pause the timer without resetting.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Reset the timer and all splits to their initial state.
    pub fn reset(&mut self) {
        self.total_time_ms = 0;
        self.running = false;
        self.current_split = 0;
        for split in &mut self.splits {
            split.time_ms = None;
            split.delta_ms = None;
        }
    }

    /// Record a split with the given name at the current total time.
    pub fn split(&mut self, name: &str) {
        if self.current_split >= self.splits.len() {
            // Auto-create the split entry if it doesn't exist yet.
            self.splits.push(Split {
                name: name.to_string(),
                time_ms: None,
                best_time_ms: self.best_splits.get(name).copied(),
                delta_ms: None,
            });
        }

        let idx = self.current_split;
        let split = &mut self.splits[idx];
        split.time_ms = Some(self.total_time_ms);

        // Compute delta from personal best for this split.
        if let Some(best) = split.best_time_ms {
            split.delta_ms = Some(self.total_time_ms as i64 - best as i64);
        }

        // Update best split time if this is a new record.
        let is_new_best = match self.best_splits.get(name) {
            Some(&best) => self.total_time_ms < best,
            None => true,
        };
        if is_new_best {
            self.best_splits
                .insert(name.to_string(), self.total_time_ms);
        }

        self.current_split += 1;
    }

    /// Advance the timer by `dt_ms` milliseconds. Call once per frame while running.
    pub fn update(&mut self, dt_ms: u64) {
        if self.running {
            self.total_time_ms += dt_ms;
        }
    }

    /// Format a time in milliseconds as `"HH:MM:SS.mmm"`.
    pub fn format_time(ms: u64) -> String {
        let total_secs = ms / 1000;
        let millis = ms % 1000;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
    }

    /// Format a delta time as `"+M:SS.mmm"` or `"-M:SS.mmm"`.
    pub fn format_delta(ms: i64) -> String {
        let sign = if ms >= 0 { '+' } else { '-' };
        let abs_ms = ms.unsigned_abs();
        let total_secs = abs_ms / 1000;
        let millis = abs_ms % 1000;
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{}{}:{:02}.{:03}", sign, minutes, seconds, millis)
    }

    /// Save the current best splits and personal best to a JSON file.
    // NOTE: Caller is responsible for sandbox validation
    pub fn save_best(&mut self, path: &str) -> Result<(), std::io::Error> {
        #[derive(Serialize)]
        struct SaveData<'a> {
            best_splits: &'a HashMap<String, u64>,
            personal_best: Option<u64>,
        }
        let data = SaveData {
            best_splits: &self.best_splits,
            personal_best: self.personal_best,
        };
        let json = serde_json::to_string_pretty(&data).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load best splits and personal best from a JSON file.
    // NOTE: Caller is responsible for sandbox validation
    pub fn load_best(&mut self, path: &str) -> Result<(), std::io::Error> {
        // Validate file size to prevent memory exhaustion attacks
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_SAVE_SIZE {
            return Err(std::io::Error::other(format!(
                "File too large: {} bytes (max {})",
                metadata.len(),
                MAX_SAVE_SIZE
            )));
        }

        #[derive(Deserialize)]
        struct SaveData {
            best_splits: HashMap<String, u64>,
            personal_best: Option<u64>,
        }
        let json = std::fs::read_to_string(path)?;
        let data: SaveData = serde_json::from_str(&json).map_err(std::io::Error::other)?;
        self.best_splits = data.best_splits;
        self.personal_best = data.personal_best;

        // Update existing splits with loaded best times.
        for split in &mut self.splits {
            split.best_time_ms = self.best_splits.get(&split.name).copied();
        }
        Ok(())
    }

    /// Returns `true` if the current run's total time beats the personal best.
    pub fn is_personal_best(&self) -> bool {
        match self.personal_best {
            Some(pb) => self.total_time_ms < pb,
            None => true,
        }
    }

    /// Build display text for the timer overlay.
    ///
    /// Returns a vec of `(split_name, formatted_time, delta_ms)` tuples for each split,
    /// followed by the overall time.
    pub fn get_display_text(&self) -> Vec<(String, String, Option<i64>)> {
        let mut result = Vec::new();
        for split in &self.splits {
            let time_str = match split.time_ms {
                Some(ms) => Self::format_time(ms),
                None => "--:--:--.---".to_string(),
            };
            result.push((split.name.clone(), time_str, split.delta_ms));
        }
        result.push((
            "Total".to_string(),
            Self::format_time(self.total_time_ms),
            self.personal_best
                .map(|pb| self.total_time_ms as i64 - pb as i64),
        ));
        result
    }
}

impl Default for SpeedrunTimer {
    fn default() -> Self {
        Self::new()
    }
}
