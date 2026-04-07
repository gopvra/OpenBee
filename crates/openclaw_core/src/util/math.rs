//! Math utility functions.

use glam::Vec2;

use crate::render::renderer::Rect;

/// Linearly interpolate between `a` and `b` by factor `t` (0.0 = a, 1.0 = b).
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp a value to the range `[min, max]`.
#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.clamp(min, max)
}

/// Convert degrees to radians.
#[inline]
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

/// Convert radians to degrees.
#[inline]
pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}

/// Check if a point lies inside a rectangle.
#[inline]
pub fn rect_contains_point(rect: &Rect, point: Vec2) -> bool {
    rect.contains(point)
}

/// Check if two rectangles overlap.
#[inline]
pub fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    a.overlaps(b)
}

/// Extension trait for `glam::Vec2` with game-useful helpers.
pub trait Vec2Ext {
    /// Rotate the vector by the given angle in radians.
    fn rotated(self, angle: f32) -> Vec2;

    /// Return the angle of the vector in radians (atan2).
    fn angle(self) -> f32;

    /// Return the vector snapped to the nearest integer coordinates.
    fn snapped(self) -> Vec2;

    /// Return the vector clamped to a maximum length.
    fn clamped_length(self, max_length: f32) -> Vec2;

    /// Linear interpolation toward another vector.
    fn lerp_to(self, target: Vec2, t: f32) -> Vec2;

    /// Move toward a target by at most `max_delta` units.
    fn move_toward(self, target: Vec2, max_delta: f32) -> Vec2;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn rotated(self, angle: f32) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        Vec2::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }

    #[inline]
    fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    #[inline]
    fn snapped(self) -> Vec2 {
        Vec2::new(self.x.round(), self.y.round())
    }

    #[inline]
    fn clamped_length(self, max_length: f32) -> Vec2 {
        let len = self.length();
        if len > max_length && len > 0.0 {
            self * (max_length / len)
        } else {
            self
        }
    }

    #[inline]
    fn lerp_to(self, target: Vec2, t: f32) -> Vec2 {
        self + (target - self) * t
    }

    #[inline]
    fn move_toward(self, target: Vec2, max_delta: f32) -> Vec2 {
        let diff = target - self;
        let dist = diff.length();
        if dist <= max_delta || dist < f32::EPSILON {
            target
        } else {
            self + diff / dist * max_delta
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < 0.001);
        assert!((lerp(0.0, 10.0, 0.0)).abs() < 0.001);
        assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_degrees_radians() {
        assert!((degrees_to_radians(180.0) - std::f32::consts::PI).abs() < 0.001);
        assert!((radians_to_degrees(std::f32::consts::PI) - 180.0).abs() < 0.001);
    }

    #[test]
    fn test_vec2_move_toward() {
        let a = Vec2::ZERO;
        let b = Vec2::new(10.0, 0.0);
        let result = a.move_toward(b, 3.0);
        assert!((result.x - 3.0).abs() < 0.001);
    }
}
