//! Replay recording and playback — captures per-tick input and event data so
//! that a game session can be deterministically replayed.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::protocol::PlayerInputData;

/// Metadata header stored at the start of a replay file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMetadata {
    /// Engine / protocol version.
    pub version: String,
    /// Level that was played.
    pub level: String,
    /// Name of the recording player.
    pub player_name: String,
    /// Date string (ISO 8601 or similar).
    pub date: String,
    /// Total duration of the replay in milliseconds.
    pub duration_ms: u64,
}

/// A single simulation frame captured during recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFrame {
    /// Simulation tick.
    pub tick: u64,
    /// Wall-clock timestamp in milliseconds since recording start.
    pub timestamp_ms: u64,
    /// Player inputs active during this frame.
    pub inputs: Vec<PlayerInputData>,
    /// Game events that occurred this frame (serialised as strings).
    pub events: Vec<String>,
}

/// Maximum number of frames in a single replay recording (roughly 3 hours at 60 fps).
const MAX_REPLAY_FRAMES: usize = 650_000;

/// Maximum allowed replay file size when loading (256 MiB).
const MAX_REPLAY_FILE_SIZE: u64 = 256 * 1024 * 1024;

/// Records a game session frame-by-frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRecorder {
    /// All captured frames.
    pub frames: Vec<ReplayFrame>,
    /// Replay header.
    pub metadata: ReplayMetadata,
    /// Whether recording is currently active.
    pub recording: bool,
}

impl ReplayRecorder {
    /// Create a new recorder in the recording state.
    pub fn new(metadata: ReplayMetadata) -> Self {
        Self {
            frames: Vec::new(),
            metadata,
            recording: true,
        }
    }

    /// Append a frame. No-op if not currently recording or if the frame limit
    /// has been reached.
    pub fn record_frame(&mut self, frame: ReplayFrame) {
        if !self.recording {
            return;
        }
        if self.frames.len() >= MAX_REPLAY_FRAMES {
            tracing::warn!(
                "Replay frame limit reached ({}), stopping recording",
                MAX_REPLAY_FRAMES
            );
            self.stop();
            return;
        }
        // Update duration from the latest timestamp.
        if frame.timestamp_ms > self.metadata.duration_ms {
            self.metadata.duration_ms = frame.timestamp_ms;
        }
        self.frames.push(frame);
    }

    /// Stop recording.
    pub fn stop(&mut self) {
        self.recording = false;
        tracing::info!(
            "Replay recording stopped — {} frames, {}ms",
            self.frames.len(),
            self.metadata.duration_ms
        );
    }

    /// Serialise the entire replay to JSON and write it to `path`.
    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        tracing::info!("Replay saved to {path}");
        Ok(())
    }

    /// Load a replay from a JSON file.
    pub fn load(path: &str) -> Result<Self> {
        // Check file size before reading to prevent DoS.
        let metadata = fs::metadata(path)?;
        anyhow::ensure!(
            metadata.len() <= MAX_REPLAY_FILE_SIZE,
            "Replay file too large: {} bytes (max {} bytes)",
            metadata.len(),
            MAX_REPLAY_FILE_SIZE
        );
        let data = fs::read_to_string(path)?;
        let recorder: Self = serde_json::from_str(&data)?;
        tracing::info!(
            "Replay loaded from {path} — {} frames",
            recorder.frames.len()
        );
        Ok(recorder)
    }

    /// Retrieve the frame for a specific tick, if it exists.
    pub fn get_frame(&self, tick: u64) -> Option<&ReplayFrame> {
        self.frames.iter().find(|f| f.tick == tick)
    }

    /// Total number of recorded frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

// ---------------------------------------------------------------------------
// Playback
// ---------------------------------------------------------------------------

/// Plays back a recorded replay frame-by-frame.
pub struct ReplayPlayer {
    /// The loaded replay data.
    pub replay: ReplayRecorder,
    /// Current playback index.
    pub current_index: usize,
    /// Whether playback is paused.
    pub paused: bool,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f64,
}

impl ReplayPlayer {
    /// Create a player from a loaded replay.
    pub fn new(replay: ReplayRecorder) -> Self {
        Self {
            replay,
            current_index: 0,
            paused: false,
            speed: 1.0,
        }
    }

    /// Advance to the next frame. Returns `None` when the replay is finished.
    pub fn next_frame(&mut self) -> Option<&ReplayFrame> {
        if self.paused {
            return self.replay.frames.get(self.current_index);
        }
        if self.current_index < self.replay.frames.len() {
            let frame = &self.replay.frames[self.current_index];
            self.current_index += 1;
            Some(frame)
        } else {
            None
        }
    }

    /// Seek to a specific tick.
    pub fn seek_to_tick(&mut self, tick: u64) {
        for (i, frame) in self.replay.frames.iter().enumerate() {
            if frame.tick >= tick {
                self.current_index = i;
                return;
            }
        }
        self.current_index = self.replay.frames.len();
    }

    /// Whether the replay has finished.
    pub fn is_finished(&self) -> bool {
        self.current_index >= self.replay.frames.len()
    }

    /// Reset playback to the beginning.
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}
