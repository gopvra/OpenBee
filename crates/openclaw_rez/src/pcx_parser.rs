//! PCX image format parser.
//!
//! Captain Claw uses standard PCX images (typically 8-bit with a 256-color
//! VGA palette appended at the end of the file).
//!
//! ## PCX header (128 bytes)
//!
//! | Offset | Size | Description               |
//! |--------|------|---------------------------|
//! | 0      | 1    | Manufacturer (0x0A)       |
//! | 1      | 1    | Version                   |
//! | 2      | 1    | Encoding (1 = RLE)        |
//! | 3      | 1    | Bits per pixel per plane  |
//! | 4–7    | 4×u16| xmin, ymin, xmax, ymax    |
//! | 8–9    | u16  | Horizontal DPI            |
//! | 10–11  | u16  | Vertical DPI              |
//! | 12–59  | 48   | EGA palette               |
//! | 60     | 1    | Reserved                  |
//! | 61     | 1    | Number of color planes    |
//! | 62–63  | u16  | Bytes per scanline plane  |
//! | 64–65  | u16  | Palette info              |
//! | 66–127 | 62   | Padding                   |

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PcxError {
    #[error("PCX data too short for header")]
    DataTooShort,

    #[error("invalid PCX manufacturer byte: {0:#04x} (expected 0x0A)")]
    InvalidManufacturer(u8),

    #[error("I/O error reading PCX: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported PCX format: {bits_per_pixel} bpp, {num_planes} planes")]
    UnsupportedFormat { bits_per_pixel: u8, num_planes: u8 },

    #[error("RLE decode error at offset {0}")]
    RleDecodeError(usize),

    #[error("missing VGA palette at end of file")]
    MissingPalette,
}

/// A parsed PCX image.
#[derive(Debug, Clone)]
pub struct PcxImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Bits per pixel per plane.
    pub bits_per_pixel: u8,
    /// Number of color planes.
    pub num_planes: u8,
    /// Decoded pixel data (palette indices for 8-bit images).
    pub pixels: Vec<u8>,
    /// Optional 256-color VGA palette (present for 8-bit images).
    pub palette: Option<Vec<[u8; 3]>>,
}

impl PcxImage {
    /// Parse a PCX image from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self, PcxError> {
        if data.len() < 128 {
            return Err(PcxError::DataTooShort);
        }

        let mut cursor = Cursor::new(data);

        let manufacturer = cursor.read_u8()?;
        if manufacturer != 0x0A {
            return Err(PcxError::InvalidManufacturer(manufacturer));
        }

        let _version = cursor.read_u8()?;
        let _encoding = cursor.read_u8()?;
        let bits_per_pixel = cursor.read_u8()?;

        let xmin = cursor.read_u16::<LittleEndian>()? as u32;
        let ymin = cursor.read_u16::<LittleEndian>()? as u32;
        let xmax = cursor.read_u16::<LittleEndian>()? as u32;
        let ymax = cursor.read_u16::<LittleEndian>()? as u32;

        let _hdpi = cursor.read_u16::<LittleEndian>()?;
        let _vdpi = cursor.read_u16::<LittleEndian>()?;

        // Skip EGA palette (48 bytes) + reserved (1 byte)
        let mut _skip = [0u8; 49];
        std::io::Read::read_exact(&mut cursor, &mut _skip)?;

        let num_planes = cursor.read_u8()?;
        let bytes_per_line = cursor.read_u16::<LittleEndian>()? as usize;
        let _palette_info = cursor.read_u16::<LittleEndian>()?;

        // Skip remaining header padding (62 bytes to reach offset 128)
        let mut _pad = [0u8; 62];
        std::io::Read::read_exact(&mut cursor, &mut _pad)?;

        let width = xmax - xmin + 1;
        let height = ymax - ymin + 1;

        // Decode RLE scanlines
        let total_bytes = bytes_per_line * (num_planes as usize) * (height as usize);
        let rle_data = &data[128..];
        let decoded = Self::decode_rle(rle_data, total_bytes)?;

        // Extract pixel data
        let pixels = if num_planes == 1 && bits_per_pixel == 8 {
            // 8-bit indexed color — one plane, one byte per pixel
            let mut px = Vec::with_capacity((width * height) as usize);
            for y in 0..height as usize {
                let line_start = y * bytes_per_line;
                for x in 0..width as usize {
                    px.push(decoded[line_start + x]);
                }
            }
            px
        } else {
            return Err(PcxError::UnsupportedFormat {
                bits_per_pixel,
                num_planes,
            });
        };

        // Read VGA palette (last 769 bytes: 0x0C marker + 256×3 RGB)
        let palette = if bits_per_pixel == 8 && num_planes == 1 && data.len() >= 769 {
            let pal_start = data.len() - 769;
            if data[pal_start] == 0x0C {
                let mut colors = Vec::with_capacity(256);
                for i in 0..256 {
                    let base = pal_start + 1 + i * 3;
                    colors.push([data[base], data[base + 1], data[base + 2]]);
                }
                Some(colors)
            } else {
                None
            }
        } else {
            None
        };

        Ok(PcxImage {
            width,
            height,
            bits_per_pixel,
            num_planes,
            pixels,
            palette,
        })
    }

    /// Decode PCX RLE-compressed data.
    fn decode_rle(data: &[u8], expected_size: usize) -> Result<Vec<u8>, PcxError> {
        let mut output = Vec::with_capacity(expected_size);
        let mut pos = 0;

        while output.len() < expected_size {
            if pos >= data.len() {
                return Err(PcxError::RleDecodeError(pos));
            }

            let byte = data[pos];
            pos += 1;

            if byte & 0xC0 == 0xC0 {
                // Run-length encoded: lower 6 bits = count
                let count = (byte & 0x3F) as usize;
                if pos >= data.len() {
                    return Err(PcxError::RleDecodeError(pos));
                }
                let value = data[pos];
                pos += 1;
                for _ in 0..count {
                    output.push(value);
                }
            } else {
                // Literal byte
                output.push(byte);
            }
        }

        output.truncate(expected_size);
        Ok(output)
    }

    /// Convert the image to RGBA pixels.
    ///
    /// Uses the embedded VGA palette if present; otherwise maps indices directly
    /// to greyscale.
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.pixels.len() * 4);

        if let Some(ref palette) = self.palette {
            for &index in &self.pixels {
                let c = palette[index as usize];
                rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
        } else {
            // Fallback: greyscale
            for &index in &self.pixels {
                rgba.extend_from_slice(&[index, index, index, 255]);
            }
        }

        rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid 8-bit PCX with a 2×2 image.
    fn make_test_pcx() -> Vec<u8> {
        let mut buf = vec![0u8; 128];
        buf[0] = 0x0A; // manufacturer
        buf[1] = 5; // version
        buf[2] = 1; // RLE encoding
        buf[3] = 8; // 8 bpp

        // xmin=0, ymin=0
        buf[4] = 0;
        buf[5] = 0;
        buf[6] = 0;
        buf[7] = 0;
        // xmax=1, ymax=1 (2×2 image)
        buf[8] = 1;
        buf[9] = 0;
        buf[10] = 1;
        buf[11] = 0;

        // num_planes is at offset 65 in the PCX header
        buf[65] = 1; // num_planes
        // bytes_per_line is at offset 66-67 (u16 LE)
        buf[66] = 2; // bytes_per_line = 2
        buf[67] = 0;

        // Pixel data: 2 scanlines of 2 bytes each (no RLE needed for small data)
        // Literal bytes: 10, 20, 30, 40
        buf.extend_from_slice(&[10, 20, 30, 40]);

        // VGA palette at end: 0x0C marker + 256*3 bytes
        buf.push(0x0C);
        for i in 0..256u16 {
            let v = (i & 0xFF) as u8;
            buf.extend_from_slice(&[v, v, v]);
        }

        buf
    }

    #[test]
    fn test_parse_pcx() {
        let data = make_test_pcx();
        let pcx = PcxImage::parse(&data).unwrap();
        assert_eq!(pcx.width, 2);
        assert_eq!(pcx.height, 2);
        assert_eq!(pcx.pixels, vec![10, 20, 30, 40]);
        assert!(pcx.palette.is_some());
    }

    #[test]
    fn test_to_rgba() {
        let data = make_test_pcx();
        let pcx = PcxImage::parse(&data).unwrap();
        let rgba = pcx.to_rgba();
        // pixel 0 = index 10 => greyscale (10,10,10,255)
        assert_eq!(&rgba[0..4], &[10, 10, 10, 255]);
    }
}
