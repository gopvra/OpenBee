//! Sprite-based animation playback.

use super::sprite::SpriteId;

/// An animation defined as a sequence of sprite frames.
#[derive(Debug, Clone)]
pub struct Animation {
    /// Ordered list of sprite frames.
    pub frames: Vec<SpriteId>,
    /// Duration of each frame in milliseconds.
    pub frame_duration_ms: u32,
    /// Whether the animation loops when it reaches the end.
    pub looping: bool,
}

impl Animation {
    /// Create a new animation.
    pub fn new(frames: Vec<SpriteId>, frame_duration_ms: u32, looping: bool) -> Self {
        Self {
            frames,
            frame_duration_ms,
            looping,
        }
    }

    /// Total duration of one cycle of the animation in milliseconds.
    pub fn total_duration_ms(&self) -> u32 {
        self.frame_duration_ms * self.frames.len() as u32
    }
}

/// Controls playback state of an animation.
#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    /// Index of the current frame within the animation's frames.
    current_frame: usize,
    /// Time elapsed within the current frame (in milliseconds).
    elapsed_ms: f32,
    /// Whether the animation is currently playing.
    playing: bool,
    /// Whether the animation has finished (non-looping only).
    finished: bool,
    /// Playback speed multiplier (1.0 = normal).
    speed: f32,
}

impl AnimationPlayer {
    /// Create a new animation player in the stopped state.
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            elapsed_ms: 0.0,
            playing: false,
            finished: false,
            speed: 1.0,
        }
    }

    /// Start or resume playback.
    pub fn play(&mut self) {
        self.playing = true;
        self.finished = false;
    }

    /// Pause playback without resetting position.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stop playback and reset to the first frame.
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_frame = 0;
        self.elapsed_ms = 0.0;
        self.finished = false;
    }

    /// Reset to the first frame and start playing.
    pub fn restart(&mut self) {
        self.stop();
        self.play();
    }

    /// Set playback speed multiplier.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Advance the animation by `dt` seconds. Call each frame with the animation definition.
    pub fn update(&mut self, animation: &Animation, dt: f32) {
        if !self.playing || self.finished || animation.frames.is_empty() {
            return;
        }

        self.elapsed_ms += dt * 1000.0 * self.speed;

        while self.elapsed_ms >= animation.frame_duration_ms as f32 {
            self.elapsed_ms -= animation.frame_duration_ms as f32;
            self.current_frame += 1;

            if self.current_frame >= animation.frames.len() {
                if animation.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = animation.frames.len() - 1;
                    self.finished = true;
                    self.playing = false;
                    return;
                }
            }
        }
    }

    /// Get the current frame's sprite ID from the animation.
    pub fn current_sprite(&self, animation: &Animation) -> Option<SpriteId> {
        animation.frames.get(self.current_frame).copied()
    }

    /// Get the current frame index.
    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// Check whether the animation is currently playing.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Check whether a non-looping animation has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new()
    }
}
