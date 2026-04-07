//! PID image format parser.
//!
//! PID is the proprietary palette-indexed image format used by Captain Claw.
//! Images can be stored raw or with simple RLE compression.
//!
//! ## Format layout
//!
//! - `flags`: u32 LE — bit 0 set means RLE compressed
//! - `width`: u32 LE
//! - `height`: u32 LE
//! - `offset_x`: i32 LE — horizontal offset for rendering
//! - `offset_y`: i32 LE — vertical offset for rendering
//! - Pixel data: either raw or RLE-compressed palette indices

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use thiserror::Error;

use crate::pal_parser::Palette;

/// Flag indicating the pixel data is RLE-compressed.
const FLAG_RLE_COMPRESSED: u32 = 1;

#[derive(Error, Debug)]
pub enum PidError {
    #[error("PID data too short for header")]
    DataTooShort,

    #[error("I/O error reading PID: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid PID dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("RLE decode error: unexpected end of data")]
    RleDecodeError,

    #[error("not enough pixel data: expected {expected}, got {actual}")]
    InsufficientPixelData { expected: usize, actual: usize },
}

/// A parsed PID image.
#[derive(Debug, Clone)]
pub struct PidImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Horizontal rendering offset.
    pub offset_x: i32,
    /// Vertical rendering offset.
    pub offset_y: i32,
    /// Palette-indexed pixel data (width × height bytes).
    pub pixels: Vec<u8>,
    /// Original flags from the file header.
    pub flags: u32,
}

impl PidImage {
    /// Parse a PID image from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self, PidError> {
        if data.len() < 20 {
            return Err(PidError::DataTooShort);
        }

        let mut cursor = Cursor::new(data);
        let flags = cursor.read_u32::<LittleEndian>()?;
        let width = cursor.read_u32::<LittleEndian>()?;
        let height = cursor.read_u32::<LittleEndian>()?;
        let offset_x = cursor.read_i32::<LittleEndian>()?;
        let offset_y = cursor.read_i32::<LittleEndian>()?;

        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(PidError::InvalidDimensions { width, height })?;

        let pixel_data = &data[20..];
        let pixels = if flags & FLAG_RLE_COMPRESSED != 0 {
            Self::decode_rle(pixel_data, pixel_count)?
        } else {
            if pixel_data.len() < pixel_count {
                return Err(PidError::InsufficientPixelData {
                    expected: pixel_count,
                    actual: pixel_data.len(),
                });
            }
            pixel_data[..pixel_count].to_vec()
        };

        Ok(PidImage {
            width,
            height,
            offset_x,
            offset_y,
            pixels,
            flags,
        })
    }

    /// Decode RLE-compressed pixel data.
    ///
    /// The RLE scheme used by Captain Claw PID files:
    /// - If the high bit of a byte is set, the lower 7 bits give a run length
    ///   of transparent pixels (index 0).
    /// - Otherwise the byte is a literal palette index.
    fn decode_rle(data: &[u8], expected_size: usize) -> Result<Vec<u8>, PidError> {
        let mut pixels = Vec::with_capacity(expected_size);
        let mut pos = 0;

        while pixels.len() < expected_size {
            if pos >= data.len() {
                return Err(PidError::RleDecodeError);
            }

            let byte = data[pos];
            pos += 1;

            if byte & 0x80 != 0 {
                // Run of transparent pixels
                let count = (byte & 0x7F) as usize;
                pixels.resize(pixels.len() + count, 0);
            } else {
                // Literal pixel
                pixels.push(byte);
            }
        }

        pixels.truncate(expected_size);
        Ok(pixels)
    }

    /// Convert the palette-indexed image to RGBA pixels.
    ///
    /// Returns a Vec of `width * height * 4` bytes in RGBA order.
    pub fn to_rgba(&self, palette: &Palette) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.pixels.len() * 4);
        for &index in &self.pixels {
            let color = palette.get_rgba(index);
            rgba.extend_from_slice(&color);
        }
        rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pid_header(flags: u32, w: u32, h: u32, ox: i32, oy: i32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&flags.to_le_bytes());
        buf.extend_from_slice(&w.to_le_bytes());
        buf.extend_from_slice(&h.to_le_bytes());
        buf.extend_from_slice(&ox.to_le_bytes());
        buf.extend_from_slice(&oy.to_le_bytes());
        buf
    }

    #[test]
    fn test_parse_raw_pid() {
        let mut data = make_pid_header(0, 2, 2, -1, -1);
        data.extend_from_slice(&[1, 2, 3, 4]);

        let pid = PidImage::parse(&data).unwrap();
        assert_eq!(pid.width, 2);
        assert_eq!(pid.height, 2);
        assert_eq!(pid.offset_x, -1);
        assert_eq!(pid.offset_y, -1);
        assert_eq!(pid.pixels, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_parse_rle_pid() {
        let mut data = make_pid_header(1, 4, 1, 0, 0);
        // 0x82 = run of 2 transparent, then literal 5, then literal 10
        data.extend_from_slice(&[0x82, 5, 10]);

        let pid = PidImage::parse(&data).unwrap();
        assert_eq!(pid.pixels, vec![0, 0, 5, 10]);
    }

    #[test]
    fn test_to_rgba() {
        let mut pal_data = vec![0u8; 768];
        pal_data[3] = 255; // color 1 = red
        let palette = Palette::parse(&pal_data).unwrap();

        let mut data = make_pid_header(0, 2, 1, 0, 0);
        data.extend_from_slice(&[0, 1]);

        let pid = PidImage::parse(&data).unwrap();
        let rgba = pid.to_rgba(&palette);
        // Index 0 -> transparent
        assert_eq!(&rgba[0..4], &[0, 0, 0, 0]);
        // Index 1 -> red, opaque
        assert_eq!(&rgba[4..8], &[255, 0, 0, 255]);
    }
}
