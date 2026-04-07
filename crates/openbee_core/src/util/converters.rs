//! Coordinate and color conversion utilities.

use crate::physics::PIXELS_PER_METER;
use crate::render::renderer::Color;

/// Convert a pixel-space value to physics meters.
#[inline]
pub fn pixels_to_meters(pixels: f32) -> f32 {
    pixels / PIXELS_PER_METER
}

/// Convert a physics meters value to pixel-space.
#[inline]
pub fn meters_to_pixels(meters: f32) -> f32 {
    meters * PIXELS_PER_METER
}

/// Create a `Color` from a hex string (e.g., "#FF0000" or "FF0000FF").
/// Supports 6-char (RGB) and 8-char (RGBA) hex strings, with or without leading `#`.
pub fn color_from_hex(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::new(r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::new(r, g, b, a))
        }
        _ => None,
    }
}

/// Create a `Color` from separate RGBA u8 values.
#[inline]
pub fn rgba_to_color(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::new(r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_meter_roundtrip() {
        let px = 150.0;
        let m = pixels_to_meters(px);
        let back = meters_to_pixels(m);
        assert!((back - px).abs() < 0.001);
    }

    #[test]
    fn test_color_from_hex() {
        let c = color_from_hex("#FF8000").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);

        let c2 = color_from_hex("00FF0080").unwrap();
        assert_eq!(c2.r, 0);
        assert_eq!(c2.g, 255);
        assert_eq!(c2.b, 0);
        assert_eq!(c2.a, 128);
    }
}
