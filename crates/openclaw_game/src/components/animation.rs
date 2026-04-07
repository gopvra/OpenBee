//! Animation component for sprite-based animation playback.

use openclaw_core::ecs::Component;
use openclaw_core::render::Animation;
use rustc_hash::FxHashMap;

/// Manages named animations for an entity.
#[derive(Debug, Clone)]
pub struct AnimationComponent {
    /// Name of the currently active animation.
    pub current_animation: String,
    /// Map from animation name to animation data.
    pub animations: FxHashMap<String, Animation>,
    /// Whether the current animation is playing.
    pub playing: bool,
    /// Playback speed multiplier.
    pub speed: f32,
}

impl AnimationComponent {
    /// Switch to a named animation, restarting if it's different from the current one.
    pub fn play(&mut self, name: &str) {
        if self.current_animation != name {
            self.current_animation = name.to_string();
            self.playing = true;
        }
    }

    /// Stop the current animation.
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Get the current animation definition, if it exists.
    pub fn current(&self) -> Option<&Animation> {
        self.animations.get(&self.current_animation)
    }
}

impl Default for AnimationComponent {
    fn default() -> Self {
        Self {
            current_animation: String::new(),
            animations: FxHashMap::default(),
            playing: false,
            speed: 1.0,
        }
    }
}

impl Component for AnimationComponent {}
