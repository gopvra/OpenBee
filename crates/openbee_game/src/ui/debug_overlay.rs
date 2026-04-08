//! Debug overlay / entity inspector for development and testing.

use openbee_core::ecs::World;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Performance metrics history.
#[derive(Debug, Clone)]
pub struct PerformanceGraph {
    pub frame_times: VecDeque<f32>,
    pub fps_history: VecDeque<f32>,
    pub max_samples: usize,
    pub update_time_ms: f32,
    pub render_time_ms: f32,
    pub physics_time_ms: f32,
    pub entity_count: usize,
    pub component_count: usize,
    pub draw_call_count: usize,
    pub particle_count: usize,
}

impl PerformanceGraph {
    /// Create a new performance graph that keeps up to `max_samples` historical values.
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_samples),
            fps_history: VecDeque::with_capacity(max_samples),
            max_samples,
            update_time_ms: 0.0,
            render_time_ms: 0.0,
            physics_time_ms: 0.0,
            entity_count: 0,
            component_count: 0,
            draw_call_count: 0,
            particle_count: 0,
        }
    }

    /// Record a new frame with the given delta time (in seconds).
    pub fn record_frame(&mut self, dt: f32) {
        let frame_ms = dt * 1000.0;
        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };

        if self.frame_times.len() >= self.max_samples {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_ms);

        if self.fps_history.len() >= self.max_samples {
            self.fps_history.pop_front();
        }
        self.fps_history.push_back(fps);
    }

    /// Record a subsystem timing by name.
    pub fn record_subsystem(&mut self, name: &str, time_ms: f32) {
        match name {
            "update" => self.update_time_ms = time_ms,
            "render" => self.render_time_ms = time_ms,
            "physics" => self.physics_time_ms = time_ms,
            _ => {}
        }
    }

    /// Average FPS over the recorded history.
    pub fn average_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.fps_history.iter().sum();
        sum / self.fps_history.len() as f32
    }

    /// Average frame time in milliseconds over the recorded history.
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.frame_times.iter().sum();
        sum / self.frame_times.len() as f32
    }

    /// Minimum FPS in the recorded history (worst performance spike).
    pub fn min_fps(&self) -> f32 {
        self.fps_history
            .iter()
            .copied()
            .reduce(f32::min)
            .unwrap_or(0.0)
    }

    /// Maximum FPS in the recorded history.
    pub fn max_fps(&self) -> f32 {
        self.fps_history
            .iter()
            .copied()
            .reduce(f32::max)
            .unwrap_or(0.0)
    }

    /// Compute the given percentile (0.0 .. 1.0) of frame times.
    /// For example `percentile_frame_time(0.99)` gives the 99th-percentile frame time.
    pub fn percentile_frame_time(&self, p: f32) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f32> = self.frame_times.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let index = ((sorted.len() as f32 * p.clamp(0.0, 1.0)) as usize).min(sorted.len() - 1);
        sorted[index]
    }
}

/// Entity inspector panel for examining individual entities at runtime.
#[derive(Debug)]
pub struct EntityInspector {
    pub selected_entity: Option<u64>,
    pub show_components: bool,
    pub show_position: bool,
    pub show_physics: bool,
    pub filter_text: String,
}

impl EntityInspector {
    /// Create a new entity inspector with nothing selected.
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            show_components: true,
            show_position: true,
            show_physics: true,
            filter_text: String::new(),
        }
    }

    /// Select an entity by id for inspection.
    pub fn select(&mut self, entity_id: u64) {
        self.selected_entity = Some(entity_id);
    }

    /// Clear the current selection.
    pub fn deselect(&mut self) {
        self.selected_entity = None;
    }

    /// Query the world for information about the currently selected entity.
    /// Returns `None` if nothing is selected or the entity does not exist.
    pub fn get_entity_info(&self, world: &World) -> Option<EntityInfo> {
        let id = self.selected_entity?;

        // Walk alive entities looking for a matching id.
        let entities = world.entities();
        let entity = entities.iter().find(|e| e.id as u64 == id)?;

        if !world.is_alive(*entity) {
            return None;
        }

        Some(EntityInfo {
            id,
            component_names: Vec::new(), // type-erased storage cannot enumerate names
            position: None,
            velocity: None,
            health: None,
            ai_state: None,
        })
    }
}

impl Default for EntityInspector {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of useful information about an entity.
#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub id: u64,
    pub component_names: Vec<String>,
    pub position: Option<(f32, f32)>,
    pub velocity: Option<(f32, f32)>,
    pub health: Option<(i32, i32)>,
    pub ai_state: Option<String>,
}

/// Debug overlay display modes, from least to most verbose.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DebugOverlayMode {
    /// Overlay is completely hidden.
    Hidden,
    /// Show only the FPS counter.
    FpsOnly,
    /// FPS + frame graph + subsystem timings.
    Performance,
    /// Click to inspect entities.
    EntityInspector,
    /// Show collision shapes as wireframes.
    PhysicsWireframe,
    /// Everything at once.
    Full,
}

const ALL_MODES: [DebugOverlayMode; 6] = [
    DebugOverlayMode::Hidden,
    DebugOverlayMode::FpsOnly,
    DebugOverlayMode::Performance,
    DebugOverlayMode::EntityInspector,
    DebugOverlayMode::PhysicsWireframe,
    DebugOverlayMode::Full,
];

/// Master debug overlay combining performance graphs, entity inspection,
/// and various visual debug aids.
pub struct DebugOverlay {
    pub mode: DebugOverlayMode,
    pub performance: PerformanceGraph,
    pub inspector: EntityInspector,
    pub show_collision_boxes: bool,
    pub show_trigger_zones: bool,
    pub show_ai_paths: bool,
    pub show_spawn_points: bool,
    pub show_grid: bool,
    pub grid_size: f32,
    /// Log messages with remaining display time in seconds.
    pub log_messages: VecDeque<(String, f32)>,
    pub max_log_messages: usize,
}

impl DebugOverlay {
    /// Create a new debug overlay, initially hidden.
    pub fn new() -> Self {
        Self {
            mode: DebugOverlayMode::Hidden,
            performance: PerformanceGraph::new(300),
            inspector: EntityInspector::new(),
            show_collision_boxes: false,
            show_trigger_zones: false,
            show_ai_paths: false,
            show_spawn_points: false,
            show_grid: false,
            grid_size: 32.0,
            log_messages: VecDeque::new(),
            max_log_messages: 20,
        }
    }

    /// Cycle to the next overlay mode.
    pub fn toggle_mode(&mut self) {
        let current_index = ALL_MODES.iter().position(|&m| m == self.mode).unwrap_or(0);
        let next_index = (current_index + 1) % ALL_MODES.len();
        self.mode = ALL_MODES[next_index];
    }

    /// Set a specific overlay mode.
    pub fn set_mode(&mut self, mode: DebugOverlayMode) {
        self.mode = mode;
    }

    /// Per-frame update: tick log message timers, update entity count, etc.
    pub fn update(&mut self, dt: f32, world: &World) {
        // Update entity count
        self.performance.entity_count = world.entity_count();

        // Tick log message timers and remove expired ones.
        for (_, time_remaining) in self.log_messages.iter_mut() {
            *time_remaining -= dt;
        }
        while self.log_messages.front().is_some_and(|(_, t)| *t <= 0.0) {
            self.log_messages.pop_front();
        }
    }

    /// Add a timed log message to the on-screen log (shown for 5 seconds).
    pub fn log(&mut self, message: &str) {
        if self.log_messages.len() >= self.max_log_messages {
            self.log_messages.pop_front();
        }
        self.log_messages.push_back((message.to_owned(), 5.0));
    }

    /// Record a frame for the performance graph.
    pub fn record_frame(&mut self, dt: f32) {
        self.performance.record_frame(dt);
    }

    /// Returns `true` if any part of the overlay should be drawn.
    pub fn is_visible(&self) -> bool {
        self.mode != DebugOverlayMode::Hidden
    }
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_graph_basics() {
        let mut graph = PerformanceGraph::new(5);
        for i in 1..=6 {
            graph.record_frame(1.0 / (i as f32 * 10.0));
        }
        // Only last 5 samples should be kept.
        assert_eq!(graph.frame_times.len(), 5);
        assert_eq!(graph.fps_history.len(), 5);
        assert!(graph.average_fps() > 0.0);
    }

    #[test]
    fn test_performance_graph_percentile() {
        let mut graph = PerformanceGraph::new(100);
        for i in 1..=100 {
            // Frame times from ~16ms to ~100ms
            graph.frame_times.push_back(i as f32);
        }
        let p99 = graph.percentile_frame_time(0.99);
        assert!(p99 >= 99.0);
        let p50 = graph.percentile_frame_time(0.5);
        assert!((p50 - 50.0).abs() < 2.0);
    }

    #[test]
    fn test_overlay_toggle() {
        let mut overlay = DebugOverlay::new();
        assert_eq!(overlay.mode, DebugOverlayMode::Hidden);
        assert!(!overlay.is_visible());

        overlay.toggle_mode();
        assert_eq!(overlay.mode, DebugOverlayMode::FpsOnly);
        assert!(overlay.is_visible());

        // Cycle all the way back to Hidden.
        for _ in 0..5 {
            overlay.toggle_mode();
        }
        assert_eq!(overlay.mode, DebugOverlayMode::Hidden);
    }

    #[test]
    fn test_overlay_log_expiry() {
        let mut overlay = DebugOverlay::new();
        overlay.log("test message");
        assert_eq!(overlay.log_messages.len(), 1);

        let world = World::new();
        // Advance time past the 5-second lifetime.
        overlay.update(6.0, &world);
        assert_eq!(overlay.log_messages.len(), 0);
    }

    #[test]
    fn test_entity_inspector_select_deselect() {
        let mut inspector = EntityInspector::new();
        assert!(inspector.selected_entity.is_none());

        inspector.select(42);
        assert_eq!(inspector.selected_entity, Some(42));

        inspector.deselect();
        assert!(inspector.selected_entity.is_none());
    }
}
