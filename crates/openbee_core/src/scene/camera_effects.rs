//! Camera effects: shake, zoom, letterbox, room transitions, and a combined
//! effects manager.

use glam::Vec2;

/// Camera shake effect driven by elapsed time and a sinusoidal offset.
#[derive(Debug, Clone)]
pub struct CameraShake {
    pub intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub frequency: f32,
    pub decay: bool,
    pub offset: Vec2,
}

impl CameraShake {
    /// Create a new shake with the given intensity (pixels) and duration (seconds).
    pub fn new(intensity: f32, duration: f32) -> Self {
        Self {
            intensity,
            duration,
            frequency: 30.0,
            elapsed: 0.0,
            decay: true,
            offset: Vec2::ZERO,
        }
    }

    /// Advance the shake by `dt` seconds and return the current frame offset.
    pub fn update(&mut self, dt: f32) -> Vec2 {
        self.elapsed += dt;
        if !self.is_active() {
            self.offset = Vec2::ZERO;
            return Vec2::ZERO;
        }

        let progress = self.elapsed / self.duration;
        let amplitude = if self.decay {
            self.intensity * (1.0 - progress)
        } else {
            self.intensity
        };

        let t = self.elapsed * self.frequency;
        self.offset = Vec2::new(t.sin() * amplitude, (t * 1.3).cos() * amplitude);
        self.offset
    }

    /// Returns `true` while the shake has remaining duration.
    pub fn is_active(&self) -> bool {
        self.elapsed < self.duration
    }
}

/// Smooth camera zoom.
#[derive(Debug, Clone)]
pub struct CameraZoom {
    pub target_zoom: f32,
    pub current_zoom: f32,
    pub zoom_speed: f32,
}

impl CameraZoom {
    pub fn new() -> Self {
        Self {
            target_zoom: 1.0,
            current_zoom: 1.0,
            zoom_speed: 2.0,
        }
    }

    /// Start zooming towards `target` at `speed` units per second.
    pub fn zoom_to(&mut self, target: f32, speed: f32) {
        self.target_zoom = target;
        self.zoom_speed = speed;
    }

    /// Advance the zoom interpolation and return the current zoom level.
    pub fn update(&mut self, dt: f32) -> f32 {
        let diff = self.target_zoom - self.current_zoom;
        if diff.abs() < 0.001 {
            self.current_zoom = self.target_zoom;
        } else {
            let step = self.zoom_speed * dt;
            self.current_zoom += diff.signum() * step.min(diff.abs());
        }
        self.current_zoom
    }
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self::new()
    }
}

/// Letterbox (cinematic bars) effect.
#[derive(Debug, Clone)]
pub struct Letterbox {
    /// Target coverage: 0.0 = no bars, 0.2 = 20 % of each edge covered.
    pub target_amount: f32,
    pub current_amount: f32,
    pub speed: f32,
    pub active: bool,
}

impl Default for Letterbox {
    fn default() -> Self {
        Self {
            target_amount: 0.0,
            current_amount: 0.0,
            speed: 2.0,
            active: false,
        }
    }
}

/// Defines a rectangular camera room with smooth transition.
#[derive(Debug, Clone)]
pub struct CameraRoom {
    /// (min_x, min_y, max_x, max_y) bounding box of the room.
    pub bounds: (f32, f32, f32, f32),
    /// How fast the camera transitions into this room.
    pub transition_speed: f32,
}

/// Combined camera effects manager.
pub struct CameraEffectsManager {
    pub shakes: Vec<CameraShake>,
    pub zoom: CameraZoom,
    pub letterbox: Letterbox,
    pub rooms: Vec<CameraRoom>,
    pub current_room: Option<usize>,
    pub smooth_follow_speed: f32,
    pub look_ahead_distance: f32,
    pub dead_zone: (f32, f32),
}

impl CameraEffectsManager {
    pub fn new() -> Self {
        Self {
            shakes: Vec::new(),
            zoom: CameraZoom::new(),
            letterbox: Letterbox::default(),
            rooms: Vec::new(),
            current_room: None,
            smooth_follow_speed: 5.0,
            look_ahead_distance: 50.0,
            dead_zone: (16.0, 16.0),
        }
    }

    /// Queue a new camera shake.
    pub fn add_shake(&mut self, intensity: f32, duration: f32) {
        self.shakes.push(CameraShake::new(intensity, duration));
    }

    /// Advance all effects and return the combined (position offset, zoom level).
    pub fn update(&mut self, dt: f32, _target: Vec2) -> (Vec2, f32) {
        // Update shakes and accumulate offset.
        let mut total_offset = Vec2::ZERO;
        for shake in &mut self.shakes {
            total_offset += shake.update(dt);
        }
        self.shakes.retain(|s| s.is_active());

        // Zoom
        let zoom = self.zoom.update(dt);

        // Letterbox interpolation
        if self.letterbox.active {
            let diff = self.letterbox.target_amount - self.letterbox.current_amount;
            if diff.abs() < 0.001 {
                self.letterbox.current_amount = self.letterbox.target_amount;
            } else {
                let step = self.letterbox.speed * dt;
                self.letterbox.current_amount += diff.signum() * step.min(diff.abs());
            }
        }

        (total_offset, zoom)
    }

    /// Switch to a specific camera room by index.
    pub fn enter_room(&mut self, room_index: usize) {
        if room_index < self.rooms.len() {
            self.current_room = Some(room_index);
        }
    }

    /// Activate or update the letterbox effect.
    pub fn set_letterbox(&mut self, amount: f32, speed: f32) {
        self.letterbox.target_amount = amount;
        self.letterbox.speed = speed;
        self.letterbox.active = true;
    }
}

impl Default for CameraEffectsManager {
    fn default() -> Self {
        Self::new()
    }
}
