//! Path elevator component for platforms that follow waypoints.

use glam::Vec2;
use openclaw_core::ecs::Component;
/// A platform that moves along a sequence of waypoints.
#[derive(Debug, Clone)]
pub struct PathElevatorComponent {
    /// Ordered list of waypoint positions in world pixels.
    pub waypoints: Vec<Vec2>,
    /// Movement speed in pixels per second.
    pub speed: f32,
    /// Index of the current target waypoint.
    pub current_waypoint: usize,
    /// Time to wait at each waypoint before moving (seconds).
    pub wait_time: f32,
    /// Whether the elevator only goes forward (true) or ping-pongs (false).
    pub one_way: bool,
    /// Internal: current wait timer.
    pub wait_timer: f32,
    /// Internal: direction of traversal (+1 forward, -1 backward).
    pub direction: i32,
}

impl Default for PathElevatorComponent {
    fn default() -> Self {
        Self {
            waypoints: Vec::new(),
            speed: 100.0,
            current_waypoint: 0,
            wait_time: 0.0,
            one_way: false,
            wait_timer: 0.0,
            direction: 1,
        }
    }
}

impl Component for PathElevatorComponent {}
