//! CLAW.REZ archive reader.
//!
//! The REZ format is a custom archive format used by Captain Claw (1997).
//! It stores all game assets (images, sounds, levels, palettes, etc.) in a
//! single file with a hierarchical directory structure.
//!
//! ## On-disk format
//!
//! ```text
//! ┌──────────────────────────┐
//! │ Header (16 bytes)        │
//! │  magic:    "REZM" (4B)   │
//! │  version:  u32 LE        │
//! │  dir_off:  u32 LE        │
//! │  dir_size: u32 LE        │
//! ├──────────────────────────┤
//! │ File data blobs          │
//! ├──────────────────────────┤
//! │ Directory tree            │
//! │  (at offset dir_off)     │
//! └──────────────────────────┘
//! ```
//!
//! Each directory entry:
//! - `entry_type`: u32 LE — 0 = file, 1 = directory
//! - `offset`: u32 LE — file data offset (files) or child dir offset (dirs)
//! - `size`: u32 LE — uncompressed size (files) or child dir size (dirs)
//! - `timestamp`: u32 LE — modification timestamp
//! - `name`: null-terminated ASCII string

use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use thiserror::Error;

const REZ_MAGIC: &[u8; 4] = b"REZM";
const ENTRY_TYPE_FILE: u32 = 0;
const ENTRY_TYPE_DIR: u32 = 1;

/// Maximum directory recursion depth to prevent stack overflow from malicious archives.
const MAX_RECURSION_DEPTH: usize = 32;

/// Maximum number of entries allowed in the archive.
const MAX_ENTRIES: usize = 500_000;

#[derive(Error, Debug)]
pub enum RezError {
    #[error("REZ file too short")]
    FileTooShort,

    #[error("invalid REZ magic: expected REZM")]
    InvalidMagic,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("directory entry at offset {offset} is out of bounds")]
    DirectoryOutOfBounds { offset: usize },

    #[error("file not found in archive: {0}")]
    FileNotFound(String),

    #[error("file data out of bounds: offset={offset}, size={size}, archive_len={archive_len}")]
    DataOutOfBounds {
        offset: u64,
        size: u64,
        archive_len: usize,
    },

    #[error("invalid UTF-8 in entry name")]
    InvalidName,

    #[error("unknown entry type {0}")]
    UnknownEntryType(u32),

    #[error("directory recursion depth exceeds maximum ({MAX_RECURSION_DEPTH})")]
    RecursionTooDeep,

    #[error("archive contains too many entries (max {MAX_ENTRIES})")]
    TooManyEntries,
}

/// A single file entry in the REZ archive.
#[derive(Debug, Clone)]
pub struct RezEntry {
    /// Full path within the archive (e.g. `CLAW/IMAGES/FRAME001.PID`).
    pub path: String,
    /// Byte offset of the file data within the archive.
    pub offset: u64,
    /// Uncompressed file size in bytes.
    pub size: u64,
    /// Compressed file size (equal to `size` when uncompressed).
    pub compressed_size: u64,
    /// Entry flags.
    pub flags: u32,
}

/// An opened CLAW.REZ archive.
#[derive(Debug)]
pub struct RezArchive {
    entries: HashMap<String, RezEntry>,
    data: Vec<u8>,
}

impl RezArchive {
    /// Open a REZ archive from a file path on disk.
    pub fn open(path: &Path) -> Result<Self, RezError> {
        let data = std::fs::read(path)?;
        Self::open_from_bytes(data)
    }

    /// Open a REZ archive from an in-memory byte buffer.
    pub fn open_from_bytes(data: Vec<u8>) -> Result<Self, RezError> {
        if data.len() < 16 {
            return Err(RezError::FileTooShort);
        }

        // Validate magic
        if &data[0..4] != REZ_MAGIC {
            return Err(RezError::InvalidMagic);
        }

        let mut cursor = Cursor::new(&data[4..12]);
        let _version = cursor.read_u32::<LittleEndian>()?;
        let dir_offset = cursor.read_u32::<LittleEndian>()? as usize;

        // Read root directory size from header
        let mut size_cursor = Cursor::new(&data[12..16]);
        let dir_size = size_cursor.read_u32::<LittleEndian>()? as usize;

        if dir_offset + dir_size > data.len() {
            return Err(RezError::DirectoryOutOfBounds { offset: dir_offset });
        }

        let mut entries = HashMap::new();
        Self::parse_directory(&data, dir_offset, dir_size, "", &mut entries, 0)?;

        Ok(RezArchive { entries, data })
    }

    /// Recursively parse a directory block in the REZ archive.
    fn parse_directory(
        data: &[u8],
        offset: usize,
        size: usize,
        prefix: &str,
        entries: &mut HashMap<String, RezEntry>,
        depth: usize,
    ) -> Result<(), RezError> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(RezError::RecursionTooDeep);
        }
        let end = offset + size;
        let mut pos = offset;

        while pos < end {
            // Need at least 16 bytes for the fixed fields before the name
            if pos + 16 > data.len() {
                break;
            }

            let mut cursor = Cursor::new(&data[pos..]);
            let entry_type = cursor.read_u32::<LittleEndian>()?;
            let entry_offset = cursor.read_u32::<LittleEndian>()?;
            let entry_size = cursor.read_u32::<LittleEndian>()?;
            let _timestamp = cursor.read_u32::<LittleEndian>()?;

            pos += 16;

            // Read null-terminated name
            let name_start = pos;
            while pos < data.len() && data[pos] != 0 {
                pos += 1;
            }
            if pos >= data.len() {
                break;
            }

            let name_bytes = &data[name_start..pos];
            let name = std::str::from_utf8(name_bytes).map_err(|_| RezError::InvalidName)?;
            pos += 1; // skip null terminator

            let full_path = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };

            match entry_type {
                ENTRY_TYPE_FILE => {
                    if entries.len() >= MAX_ENTRIES {
                        return Err(RezError::TooManyEntries);
                    }
                    let normalized = full_path.to_uppercase();
                    entries.insert(
                        normalized.clone(),
                        RezEntry {
                            path: full_path,
                            offset: entry_offset as u64,
                            size: entry_size as u64,
                            compressed_size: entry_size as u64,
                            flags: 0,
                        },
                    );
                }
                ENTRY_TYPE_DIR => {
                    Self::parse_directory(
                        data,
                        entry_offset as usize,
                        entry_size as usize,
                        &full_path,
                        entries,
                        depth + 1,
                    )?;
                }
                other => {
                    tracing::warn!(
                        "unknown REZ entry type {} for '{}', skipping",
                        other,
                        full_path
                    );
                }
            }
        }

        Ok(())
    }

    /// List all file paths in the archive.
    pub fn list_files(&self) -> Vec<&str> {
        self.entries.values().map(|e| e.path.as_str()).collect()
    }

    /// List files and subdirectories directly under the given directory path.
    ///
    /// The directory path should use `/` separators and is matched
    /// case-insensitively. Returns only the immediate children (file names
    /// or subdirectory names), not full paths.
    pub fn list_directory(&self, dir: &str) -> Vec<&str> {
        let normalized = dir.trim_end_matches('/').to_uppercase();
        let prefix = if normalized.is_empty() {
            String::new()
        } else {
            format!("{}/", normalized)
        };

        let mut results: Vec<&str> = Vec::new();
        let mut seen_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

        for entry in self.entries.values() {
            let upper = entry.path.to_uppercase();
            if prefix.is_empty() {
                // Root listing: return top-level names
                if let Some(slash_pos) = upper.find('/') {
                    let dir_name = &entry.path[..slash_pos];
                    let key = upper[..slash_pos].to_string();
                    if seen_dirs.insert(key) {
                        results.push(dir_name);
                    }
                } else {
                    results.push(&entry.path);
                }
            } else if upper.starts_with(&prefix) {
                let remainder = &entry.path[prefix.len()..];
                if let Some(slash_pos) = remainder.find('/') {
                    let dir_name = &remainder[..slash_pos];
                    let key = dir_name.to_uppercase();
                    if seen_dirs.insert(key) {
                        // Return a reference into the entry path
                        let start = prefix.len();
                        let end = start + slash_pos;
                        results.push(&entry.path[start..end]);
                    }
                } else {
                    results.push(remainder);
                }
            }
        }

        results
    }

    /// Read the raw bytes of a file from the archive.
    ///
    /// The path is matched case-insensitively using `/` separators.
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, RezError> {
        let normalized = path.to_uppercase();
        let entry = self
            .entries
            .get(&normalized)
            .ok_or_else(|| RezError::FileNotFound(path.to_string()))?;

        let start = entry.offset as usize;
        let end = start + entry.size as usize;

        if end > self.data.len() {
            return Err(RezError::DataOutOfBounds {
                offset: entry.offset,
                size: entry.size,
                archive_len: self.data.len(),
            });
        }

        Ok(self.data[start..end].to_vec())
    }

    /// Check if a file exists in the archive (case-insensitive).
    pub fn contains(&self, path: &str) -> bool {
        self.entries.contains_key(&path.to_uppercase())
    }

    /// Get the entry metadata for a file (case-insensitive lookup).
    pub fn entry(&self, path: &str) -> Option<&RezEntry> {
        self.entries.get(&path.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal REZ archive in memory for testing.
    fn make_test_rez() -> Vec<u8> {
        // File data: "hello" at offset 16
        let file_data = b"hello";
        let file_offset: u32 = 16;
        let file_size: u32 = file_data.len() as u32;

        // Directory starts after header + file data
        let dir_offset: u32 = 16 + file_size;

        // Build directory entry for a file named "TEST.TXT"
        let name = b"TEST.TXT\0";
        let dir_entry_size = 16 + name.len(); // type + offset + size + timestamp + name

        let mut buf = Vec::new();

        // Header
        buf.extend_from_slice(b"REZM"); // magic
        buf.extend_from_slice(&1u32.to_le_bytes()); // version
        buf.extend_from_slice(&dir_offset.to_le_bytes()); // dir offset
        buf.extend_from_slice(&(dir_entry_size as u32).to_le_bytes()); // dir size

        // File data
        buf.extend_from_slice(file_data);

        // Directory entry (file)
        buf.extend_from_slice(&ENTRY_TYPE_FILE.to_le_bytes()); // type = file
        buf.extend_from_slice(&file_offset.to_le_bytes()); // data offset
        buf.extend_from_slice(&file_size.to_le_bytes()); // size
        buf.extend_from_slice(&0u32.to_le_bytes()); // timestamp
        buf.extend_from_slice(name); // name

        buf
    }

    #[test]
    fn test_open_and_read() {
        let data = make_test_rez();
        let archive = RezArchive::open_from_bytes(data).unwrap();

        assert!(archive.contains("TEST.TXT"));
        assert!(archive.contains("test.txt")); // case-insensitive

        let content = archive.read_file("TEST.TXT").unwrap();
        assert_eq!(content, b"hello");
    }

    #[test]
    fn test_list_files() {
        let data = make_test_rez();
        let archive = RezArchive::open_from_bytes(data).unwrap();

        let files = archive.list_files();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_entry_metadata() {
        let data = make_test_rez();
        let archive = RezArchive::open_from_bytes(data).unwrap();

        let entry = archive.entry("TEST.TXT").unwrap();
        assert_eq!(entry.size, 5);
        assert_eq!(entry.offset, 16);
    }

    #[test]
    fn test_invalid_magic() {
        let mut data = make_test_rez();
        data[0] = b'X';
        assert!(RezArchive::open_from_bytes(data).is_err());
    }

    #[test]
    fn test_file_not_found() {
        let data = make_test_rez();
        let archive = RezArchive::open_from_bytes(data).unwrap();
        assert!(archive.read_file("NONEXISTENT.TXT").is_err());
    }
}
