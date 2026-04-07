//! Debug drawing trait for physics visualization.

use glam::Vec2;
use crate::render::renderer::Color;

/// Trait for debug-drawing physics shapes. Implement this on your renderer to visualize
/// colliders, joints, and contact points.
pub trait PhysicsDebugDrawer {
    /// Draw a circle outline at the given center with the given radius (in pixels).
    fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color);

    /// Draw a rectangle outline defined by the given center and half-extents (in pixels).
    fn draw_rect(&mut self, center: Vec2, half_width: f32, half_height: f32, color: Color);

    /// Draw a line segment between two points (in pixels).
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color);

    /// Draw a point marker at the given position.
    fn draw_point(&mut self, position: Vec2, size: f32, color: Color);
}
