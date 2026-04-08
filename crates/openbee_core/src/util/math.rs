//! Math utility functions.

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
}
