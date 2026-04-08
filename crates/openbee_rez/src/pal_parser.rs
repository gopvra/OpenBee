//! PAL palette format parser.
//!
//! Captain Claw uses 256-color palettes stored as 768 bytes (256 × 3 RGB).
//! Index 0 is conventionally treated as the transparent color.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PalError {
    #[error("palette data too short: expected at least 768 bytes, got {0}")]
    DataTooShort(usize),
}

/// A 256-color RGB palette.
#[derive(Debug, Clone)]
pub struct Palette {
    /// 256 RGB color entries.
    pub colors: [[u8; 3]; 256],
}

impl Palette {
    /// Parse a palette from raw bytes.
    ///
    /// Expects at least 768 bytes (256 colors × 3 bytes each).
    pub fn parse(data: &[u8]) -> Result<Self, PalError> {
        if data.len() < 768 {
            return Err(PalError::DataTooShort(data.len()));
        }

        let mut colors = [[0u8; 3]; 256];
        for (i, color) in colors.iter_mut().enumerate() {
            let base = i * 3;
            *color = [data[base], data[base + 1], data[base + 2]];
        }

        Ok(Palette { colors })
    }

    /// Get the RGB color at the given palette index.
    pub fn get_color(&self, index: u8) -> (u8, u8, u8) {
        // SAFETY: index is u8 (0..255); colors array has 256 entries.
        let c = self.colors[index as usize];
        (c[0], c[1], c[2])
    }

    /// Get the RGBA color at the given palette index.
    ///
    /// Index 0 is treated as transparent (alpha = 0). All other indices
    /// have alpha = 255.
    pub fn get_rgba(&self, index: u8) -> [u8; 4] {
        if index == 0 {
            [0, 0, 0, 0]
        } else {
            // SAFETY: index is u8 (0..255); colors array has 256 entries.
            let c = self.colors[index as usize];
            [c[0], c[1], c[2], 255]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_palette() {
        let mut data = vec![0u8; 768];
        // Set color 1 to red
        data[3] = 255;
        data[4] = 0;
        data[5] = 0;
        // Set color 255 to white
        data[765] = 255;
        data[766] = 255;
        data[767] = 255;

        let pal = Palette::parse(&data).unwrap();
        assert_eq!(pal.get_color(0), (0, 0, 0));
        assert_eq!(pal.get_color(1), (255, 0, 0));
        assert_eq!(pal.get_color(255), (255, 255, 255));
    }

    #[test]
    fn test_transparent_index_zero() {
        let data = vec![0u8; 768];
        let pal = Palette::parse(&data).unwrap();
        assert_eq!(pal.get_rgba(0), [0, 0, 0, 0]);
        assert_eq!(pal.get_rgba(1)[3], 255);
    }

    #[test]
    fn test_data_too_short() {
        let data = vec![0u8; 100];
        assert!(Palette::parse(&data).is_err());
    }
}
