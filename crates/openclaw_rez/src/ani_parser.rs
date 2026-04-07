//! ANI animation format parser.
//!
//! ANI files describe frame-based animations for Captain Claw sprites.
//! Each frame references a PID image file and includes timing and offset data.
//!
//! ## Format layout
//!
//! - `num_frames`: u32 LE — number of animation frames
//! - For each frame:
//!   - `event_flag`: u32 LE — sound or event trigger flags
//!   - `duration_ms`: u32 LE — frame display duration in milliseconds
//!   - `offset_x`: i32 LE — horizontal rendering offset
//!   - `offset_y`: i32 LE — vertical rendering offset
//!   - `image_file_len`: u32 LE — length of the image file path string
//!   - `image_file`: `image_file_len` bytes — null-terminated path to PID file

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AniError {
    #[error("ANI data too short")]
    DataTooShort,

    #[error("I/O error reading ANI: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid UTF-8 in frame image path")]
    InvalidString,
}

/// A parsed ANI animation file.
#[derive(Debug, Clone)]
pub struct AniFile {
    /// The animation frames in playback order.
    pub frames: Vec<AniFrame>,
}

/// A single animation frame.
#[derive(Debug, Clone)]
pub struct AniFrame {
    /// How long this frame should be displayed, in milliseconds.
    pub duration_ms: u32,
    /// Path to the PID image file for this frame.
    pub image_file: String,
    /// Event/sound trigger flags.
    pub event_flag: u32,
    /// Horizontal rendering offset.
    pub offset_x: i32,
    /// Vertical rendering offset.
    pub offset_y: i32,
}

impl AniFile {
    /// Parse an ANI animation from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self, AniError> {
        if data.len() < 4 {
            return Err(AniError::DataTooShort);
        }

        let mut cursor = Cursor::new(data);
        let num_frames = cursor.read_u32::<LittleEndian>()?;

        let mut frames = Vec::with_capacity(num_frames as usize);

        for _ in 0..num_frames {
            let event_flag = cursor.read_u32::<LittleEndian>()?;
            let duration_ms = cursor.read_u32::<LittleEndian>()?;
            let offset_x = cursor.read_i32::<LittleEndian>()?;
            let offset_y = cursor.read_i32::<LittleEndian>()?;

            let path_len = cursor.read_u32::<LittleEndian>()? as usize;
            let mut path_buf = vec![0u8; path_len];
            cursor.read_exact(&mut path_buf)?;

            // Strip null terminator if present
            if let Some(pos) = path_buf.iter().position(|&b| b == 0) {
                path_buf.truncate(pos);
            }

            let image_file =
                String::from_utf8(path_buf).map_err(|_| AniError::InvalidString)?;

            frames.push(AniFrame {
                duration_ms,
                image_file,
                event_flag,
                offset_x,
                offset_y,
            });
        }

        Ok(AniFile { frames })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_ani(frames: &[(&str, u32, u32, i32, i32)]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(frames.len() as u32).to_le_bytes());

        for &(path, event, dur, ox, oy) in frames {
            buf.extend_from_slice(&event.to_le_bytes());
            buf.extend_from_slice(&dur.to_le_bytes());
            buf.extend_from_slice(&ox.to_le_bytes());
            buf.extend_from_slice(&oy.to_le_bytes());
            let path_bytes = path.as_bytes();
            // +1 for null terminator
            buf.extend_from_slice(&((path_bytes.len() + 1) as u32).to_le_bytes());
            buf.extend_from_slice(path_bytes);
            buf.push(0); // null terminator
        }

        buf
    }

    #[test]
    fn test_parse_ani() {
        let data = build_ani(&[
            ("CLAW/IMAGES/FRAME001.PID", 0, 100, -5, 10),
            ("CLAW/IMAGES/FRAME002.PID", 1, 150, 0, 0),
        ]);

        let ani = AniFile::parse(&data).unwrap();
        assert_eq!(ani.frames.len(), 2);

        assert_eq!(ani.frames[0].image_file, "CLAW/IMAGES/FRAME001.PID");
        assert_eq!(ani.frames[0].duration_ms, 100);
        assert_eq!(ani.frames[0].event_flag, 0);
        assert_eq!(ani.frames[0].offset_x, -5);
        assert_eq!(ani.frames[0].offset_y, 10);

        assert_eq!(ani.frames[1].image_file, "CLAW/IMAGES/FRAME002.PID");
        assert_eq!(ani.frames[1].duration_ms, 150);
        assert_eq!(ani.frames[1].event_flag, 1);
    }

    #[test]
    fn test_empty_ani() {
        let data = 0u32.to_le_bytes();
        let ani = AniFile::parse(&data).unwrap();
        assert!(ani.frames.is_empty());
    }
}
