//! Tween/easing engine with 30+ easing functions and a tween manager.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Easing types
// ---------------------------------------------------------------------------

/// Standard easing function types (Penner easing family).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EaseType {
    Linear,
    SineIn,
    SineOut,
    SineInOut,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    QuintIn,
    QuintOut,
    QuintInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    CircIn,
    CircOut,
    CircInOut,
    BackIn,
    BackOut,
    BackInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

// ---------------------------------------------------------------------------
// Easing functions — all 31 variants (Penner formulas)
// ---------------------------------------------------------------------------

/// Evaluate an easing function at normalised time `t` in `[0.0, 1.0]`.
pub fn ease(t: f32, ease_type: EaseType) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match ease_type {
        EaseType::Linear => t,

        // --- Sine ---
        EaseType::SineIn => 1.0 - ((t * PI) / 2.0).cos(),
        EaseType::SineOut => ((t * PI) / 2.0).sin(),
        EaseType::SineInOut => -((PI * t).cos() - 1.0) / 2.0,

        // --- Quad ---
        EaseType::QuadIn => t * t,
        EaseType::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
        EaseType::QuadInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }

        // --- Cubic ---
        EaseType::CubicIn => t * t * t,
        EaseType::CubicOut => 1.0 - (1.0 - t).powi(3),
        EaseType::CubicInOut => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }

        // --- Quart ---
        EaseType::QuartIn => t * t * t * t,
        EaseType::QuartOut => 1.0 - (1.0 - t).powi(4),
        EaseType::QuartInOut => {
            if t < 0.5 {
                8.0 * t * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
            }
        }

        // --- Quint ---
        EaseType::QuintIn => t * t * t * t * t,
        EaseType::QuintOut => 1.0 - (1.0 - t).powi(5),
        EaseType::QuintInOut => {
            if t < 0.5 {
                16.0 * t * t * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
            }
        }

        // --- Expo ---
        EaseType::ExpoIn => {
            if t == 0.0 {
                0.0
            } else {
                (2.0_f32).powf(10.0 * t - 10.0)
            }
        }
        EaseType::ExpoOut => {
            if t == 1.0 {
                1.0
            } else {
                1.0 - (2.0_f32).powf(-10.0 * t)
            }
        }
        EaseType::ExpoInOut => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                (2.0_f32).powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - (2.0_f32).powf(-20.0 * t + 10.0)) / 2.0
            }
        }

        // --- Circ ---
        EaseType::CircIn => 1.0 - (1.0 - t * t).sqrt(),
        EaseType::CircOut => (1.0 - (t - 1.0) * (t - 1.0)).sqrt(),
        EaseType::CircInOut => {
            if t < 0.5 {
                (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
            } else {
                ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
            }
        }

        // --- Back ---
        EaseType::BackIn => {
            let c1 = 1.70158_f32;
            let c3 = c1 + 1.0;
            c3 * t * t * t - c1 * t * t
        }
        EaseType::BackOut => {
            let c1 = 1.70158_f32;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        EaseType::BackInOut => {
            let c1 = 1.70158_f32;
            let c2 = c1 * 1.525;
            if t < 0.5 {
                ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
            } else {
                ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
            }
        }

        // --- Elastic ---
        EaseType::ElasticIn => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                let c4 = (2.0 * PI) / 3.0;
                -(2.0_f32).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * c4).sin()
            }
        }
        EaseType::ElasticOut => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                let c4 = (2.0 * PI) / 3.0;
                (2.0_f32).powf(-10.0 * t) * ((10.0 * t - 0.75) * c4).sin() + 1.0
            }
        }
        EaseType::ElasticInOut => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                let c5 = (2.0 * PI) / 4.5;
                if t < 0.5 {
                    -((2.0_f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                } else {
                    ((2.0_f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                        + 1.0
                }
            }
        }

        // --- Bounce ---
        EaseType::BounceIn => 1.0 - bounce_out(1.0 - t),
        EaseType::BounceOut => bounce_out(t),
        EaseType::BounceInOut => {
            if t < 0.5 {
                (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
            } else {
                (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
            }
        }
    }
}

/// Helper for bounce easing (Penner formula).
fn bounce_out(t: f32) -> f32 {
    let n1 = 7.5625_f32;
    let d1 = 2.75_f32;

    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

// ---------------------------------------------------------------------------
// Loop mode
// ---------------------------------------------------------------------------

/// How a tween behaves when it reaches its end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopMode {
    /// Play once and stop.
    None,
    /// Restart from the beginning.
    Loop,
    /// Alternate direction each cycle.
    PingPong,
}

// ---------------------------------------------------------------------------
// Tween
// ---------------------------------------------------------------------------

/// A single tween that interpolates a float value over time.
#[derive(Debug, Clone)]
pub struct Tween {
    pub id: u64,
    pub from: f32,
    pub to: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub ease_type: EaseType,
    pub is_playing: bool,
    pub is_complete: bool,
    pub loop_mode: LoopMode,
    pub loop_count: i32,
    pub current_loop: i32,
    pub delay: f32,
    pub delay_remaining: f32,
}

impl Tween {
    /// Create a new tween.
    pub fn new(from: f32, to: f32, duration: f32, ease_type: EaseType) -> Self {
        Self {
            id: 0,
            from,
            to,
            duration: duration.max(0.0001),
            elapsed: 0.0,
            ease_type,
            is_playing: true,
            is_complete: false,
            loop_mode: LoopMode::None,
            loop_count: 0,
            current_loop: 0,
            delay: 0.0,
            delay_remaining: 0.0,
        }
    }

    /// Add an initial delay before the tween starts (builder pattern).
    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self.delay_remaining = delay;
        self
    }

    /// Set loop behaviour (builder pattern).
    pub fn with_loop(mut self, mode: LoopMode, count: i32) -> Self {
        self.loop_mode = mode;
        self.loop_count = count;
        self
    }

    /// Advance the tween by `dt` seconds and return the current interpolated value.
    pub fn update(&mut self, dt: f32) -> f32 {
        if !self.is_playing || self.is_complete {
            return self.current_value();
        }

        // Handle delay.
        if self.delay_remaining > 0.0 {
            self.delay_remaining -= dt;
            if self.delay_remaining > 0.0 {
                return self.from;
            }
            // Carry over excess dt.
            let excess = -self.delay_remaining;
            self.delay_remaining = 0.0;
            self.elapsed += excess;
        } else {
            self.elapsed += dt;
        }

        if self.elapsed >= self.duration {
            match self.loop_mode {
                LoopMode::None => {
                    self.elapsed = self.duration;
                    self.is_complete = true;
                    self.is_playing = false;
                }
                LoopMode::Loop => {
                    self.current_loop += 1;
                    if self.loop_count >= 0 && self.current_loop >= self.loop_count {
                        self.elapsed = self.duration;
                        self.is_complete = true;
                        self.is_playing = false;
                    } else {
                        self.elapsed -= self.duration;
                    }
                }
                LoopMode::PingPong => {
                    self.current_loop += 1;
                    if self.loop_count >= 0 && self.current_loop >= self.loop_count {
                        self.elapsed = self.duration;
                        self.is_complete = true;
                        self.is_playing = false;
                    } else {
                        self.elapsed -= self.duration;
                        // Swap direction.
                        std::mem::swap(&mut self.from, &mut self.to);
                    }
                }
            }
        }

        self.current_value()
    }

    /// Normalised progress (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Reset the tween to its initial state.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.is_playing = true;
        self.is_complete = false;
        self.current_loop = 0;
        self.delay_remaining = self.delay;
    }

    /// Whether the tween has finished playing.
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    /// Compute the current interpolated value without advancing time.
    fn current_value(&self) -> f32 {
        let t = ease(self.progress(), self.ease_type);
        self.from + (self.to - self.from) * t
    }
}

// ---------------------------------------------------------------------------
// TweenSequence
// ---------------------------------------------------------------------------

/// A sequence of tweens that play one after another.
#[derive(Debug, Clone)]
pub struct TweenSequence {
    pub tweens: Vec<Tween>,
    pub current_index: usize,
    pub is_complete: bool,
}

impl TweenSequence {
    /// Create an empty sequence.
    pub fn new() -> Self {
        Self {
            tweens: Vec::new(),
            current_index: 0,
            is_complete: false,
        }
    }

    /// Append a tween to the sequence (builder pattern).
    pub fn then(mut self, tween: Tween) -> Self {
        self.tweens.push(tween);
        self
    }

    /// Append a pause (builder pattern).
    pub fn then_wait(self, seconds: f32) -> Self {
        let wait = Tween::new(0.0, 0.0, seconds, EaseType::Linear);
        self.then(wait)
    }

    /// Advance the active tween. Returns the current value of the active tween,
    /// or `None` if the sequence is complete.
    pub fn update(&mut self, dt: f32) -> Option<f32> {
        if self.is_complete {
            return None;
        }

        if self.current_index >= self.tweens.len() {
            self.is_complete = true;
            return None;
        }

        let mut remaining_dt = dt;

        while remaining_dt > 0.0 && self.current_index < self.tweens.len() {
            let tween = &mut self.tweens[self.current_index];
            let value = tween.update(remaining_dt);

            if tween.is_complete() {
                // Calculate leftover time.
                let used = tween.duration - (tween.elapsed - remaining_dt).max(0.0);
                remaining_dt -= used.max(0.0);
                if remaining_dt < 0.0 {
                    remaining_dt = 0.0;
                }
                self.current_index += 1;

                if self.current_index >= self.tweens.len() {
                    self.is_complete = true;
                    return Some(value);
                }
            } else {
                return Some(value);
            }
        }

        // Fallback: return last value.
        if self.current_index > 0 {
            let prev = &self.tweens[self.current_index - 1];
            Some(prev.to)
        } else {
            None
        }
    }

    /// Check if the entire sequence has finished.
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    /// Reset the entire sequence.
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.is_complete = false;
        for tween in &mut self.tweens {
            tween.reset();
        }
    }
}

impl Default for TweenSequence {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// TweenManager
// ---------------------------------------------------------------------------

/// Manages multiple named tweens and sequences.
pub struct TweenManager {
    next_id: u64,
    tweens: HashMap<u64, Tween>,
    sequences: HashMap<u64, TweenSequence>,
}

impl TweenManager {
    /// Create a new, empty tween manager.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            tweens: HashMap::new(),
            sequences: HashMap::new(),
        }
    }

    /// Add a tween and return its unique id.
    pub fn add(&mut self, mut tween: Tween) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        tween.id = id;
        self.tweens.insert(id, tween);
        id
    }

    /// Add a tween sequence and return its unique id.
    pub fn add_sequence(&mut self, seq: TweenSequence) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.sequences.insert(id, seq);
        id
    }

    /// Get the current value of a tween or sequence by id.
    pub fn get(&self, id: u64) -> Option<f32> {
        if let Some(tween) = self.tweens.get(&id) {
            Some(tween.from + (tween.to - tween.from) * ease(tween.progress(), tween.ease_type))
        } else if let Some(seq) = self.sequences.get(&id) {
            if seq.current_index < seq.tweens.len() {
                let t = &seq.tweens[seq.current_index];
                Some(t.from + (t.to - t.from) * ease(t.progress(), t.ease_type))
            } else if !seq.tweens.is_empty() {
                let last = seq.tweens.last().unwrap();
                Some(last.to)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if a tween or sequence is complete.
    pub fn is_complete(&self, id: u64) -> bool {
        if let Some(tween) = self.tweens.get(&id) {
            tween.is_complete
        } else if let Some(seq) = self.sequences.get(&id) {
            seq.is_complete
        } else {
            true // non-existent tweens are considered complete
        }
    }

    /// Cancel and remove a tween or sequence.
    pub fn cancel(&mut self, id: u64) {
        self.tweens.remove(&id);
        self.sequences.remove(&id);
    }

    /// Advance all active tweens and sequences by `dt`.
    pub fn update(&mut self, dt: f32) {
        // Update individual tweens.
        for tween in self.tweens.values_mut() {
            if tween.is_playing && !tween.is_complete {
                tween.update(dt);
            }
        }
        // Remove completed tweens.
        self.tweens.retain(|_, t| !t.is_complete);

        // Update sequences.
        for seq in self.sequences.values_mut() {
            if !seq.is_complete {
                seq.update(dt);
            }
        }
        // Remove completed sequences.
        self.sequences.retain(|_, s| !s.is_complete);
    }

    /// Remove all tweens and sequences.
    pub fn clear(&mut self) {
        self.tweens.clear();
        self.sequences.clear();
    }
}

impl Default for TweenManager {
    fn default() -> Self {
        Self::new()
    }
}
