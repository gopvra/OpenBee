//! WWD (World Wide Data) level format parser.
//!
//! WWD files describe Captain Claw levels including tile maps, objects,
//! parallax planes, and metadata. The format consists of:
//!
//! 1. A fixed-size header with level metadata
//! 2. Plane descriptors (tile maps + properties)
//! 3. Tile index data per plane
//! 4. Object data per plane
//! 5. A string table referenced by offsets in the header/descriptors

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WwdError {
    #[error("WWD data too short")]
    DataTooShort,

    #[error("I/O error reading WWD: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid string offset {0}")]
    InvalidStringOffset(u32),

    #[error("plane descriptor out of bounds at index {0}")]
    PlaneOutOfBounds(usize),

    #[error("invalid UTF-8 in string table")]
    InvalidString,

    #[error("tile data out of bounds for plane '{0}'")]
    TileDataOutOfBounds(String),

    #[error("object data out of bounds for plane '{0}'")]
    ObjectDataOutOfBounds(String),

    #[error("too many planes: {0} (max {MAX_PLANES})")]
    TooManyPlanes(u32),

    #[error("too many objects in plane '{plane}': {count} (max {MAX_OBJECTS_PER_PLANE})")]
    TooManyObjects { plane: String, count: u32 },

    #[error("tile count too large for plane '{plane}': {count} (max {MAX_TILES_PER_PLANE})")]
    TooManyTiles { plane: String, count: usize },
}

/// Rectangle used for hit boxes, clip rects, etc.
#[derive(Debug, Clone, Default)]
pub struct WwdRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl WwdRect {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self, WwdError> {
        Ok(WwdRect {
            left: cursor.read_i32::<LittleEndian>()?,
            top: cursor.read_i32::<LittleEndian>()?,
            right: cursor.read_i32::<LittleEndian>()?,
            bottom: cursor.read_i32::<LittleEndian>()?,
        })
    }
}

/// Level header containing global metadata.
#[derive(Debug, Clone)]
pub struct WwdHeader {
    /// Header flags.
    pub flags: u32,
    /// Level display name.
    pub level_name: String,
    /// Author name.
    pub author: String,
    /// Creation date string.
    pub birth: String,
    /// REZ archive file path.
    pub rez_file: String,
    /// Base image directory within the REZ.
    pub image_dir: String,
    /// Palette resource path.
    pub pal_rez: String,
    /// Player start X position.
    pub start_x: i32,
    /// Player start Y position.
    pub start_y: i32,
    /// Number of planes in the level.
    pub num_planes: u32,
    /// Executable file name.
    pub exe_file: String,
}

/// A parallax/tile plane within a level.
#[derive(Debug, Clone)]
pub struct WwdPlane {
    /// Plane display name.
    pub name: String,
    /// Plane flags.
    pub flags: u32,
    /// Tile width in pixels.
    pub tile_width: u32,
    /// Tile height in pixels.
    pub tile_height: u32,
    /// Horizontal parallax scroll percentage (100 = 1:1 with camera).
    pub movement_x_percent: u32,
    /// Vertical parallax scroll percentage.
    pub movement_y_percent: u32,
    /// Background fill color.
    pub fill_color: u32,
    /// Z-order coordinate.
    pub z_coord: i32,
    /// Number of tiles horizontally.
    pub tiles_wide: u32,
    /// Number of tiles vertically.
    pub tiles_high: u32,
    /// Tile index array (tiles_wide × tiles_high).
    pub tiles: Vec<u32>,
    /// Image set paths used by this plane.
    pub image_sets: Vec<String>,
    /// Objects placed on this plane.
    pub objects: Vec<WwdObject>,
}

/// A game object placed within a level plane.
#[derive(Debug, Clone)]
pub struct WwdObject {
    /// Unique object identifier.
    pub id: u32,
    /// Object name.
    pub name: String,
    /// Logic class name (e.g. "CaptainClaw", "Officer", "Elevator").
    pub logic: String,
    /// Image set for this object.
    pub image_set: String,
    /// Animation name.
    pub animation: String,
    /// World X position.
    pub x: i32,
    /// World Y position.
    pub y: i32,
    /// Z depth.
    pub z: i32,
    /// Generic integer value.
    pub i_value: i32,
    /// Horizontal speed.
    pub speed_x: i32,
    /// Vertical speed.
    pub speed_y: i32,
    /// Damage inflicted.
    pub damage: i32,
    /// AI smarts parameter.
    pub smarts: i32,
    /// Hit points.
    pub health: i32,
    /// Score value.
    pub score: i32,
    /// Point value.
    pub points: i32,
    /// Powerup type.
    pub powerup: i32,
    /// Facing direction.
    pub direction: i32,
    /// Hit detection rectangle.
    pub hit_rect: WwdRect,
    /// Attack detection rectangle.
    pub attack_rect: WwdRect,
    /// Clipping rectangle.
    pub clip_rect: WwdRect,
    /// User-defined rectangles.
    pub user_rects: Vec<WwdRect>,
    /// User-defined integer values.
    pub user_values: Vec<i32>,
    /// Movement bounding rectangle.
    pub move_rect: WwdRect,
    /// Drawing flags.
    pub draw_flags: u32,
    /// User flags.
    pub user_flags: u32,
}

/// A complete parsed WWD level file.
#[derive(Debug, Clone)]
pub struct WwdFile {
    /// Level header with global metadata.
    pub header: WwdHeader,
    /// Parallax/tile planes.
    pub planes: Vec<WwdPlane>,
}

/// Size of the fixed portion of the WWD header in bytes.
const WWD_HEADER_SIZE: usize = 1524;

/// Size of each plane descriptor in bytes.
const PLANE_DESCRIPTOR_SIZE: usize = 160;

/// Maximum number of planes allowed in a WWD file (original game has ~4).
const MAX_PLANES: u32 = 64;

/// Maximum number of objects per plane.
const MAX_OBJECTS_PER_PLANE: u32 = 100_000;

/// Maximum tile count per plane (width * height).
const MAX_TILES_PER_PLANE: usize = 10_000_000;

/// Read a null-terminated string from the string table at the given offset.
fn read_string_table_entry(data: &[u8], offset: u32) -> Result<String, WwdError> {
    let start = offset as usize;
    if start >= data.len() {
        return Err(WwdError::InvalidStringOffset(offset));
    }

    let end = data[start..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| start + p)
        .unwrap_or(data.len());

    String::from_utf8(data[start..end].to_vec()).map_err(|_| WwdError::InvalidString)
}

impl WwdFile {
    /// Parse a WWD level file from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self, WwdError> {
        if data.len() < WWD_HEADER_SIZE {
            return Err(WwdError::DataTooShort);
        }

        let mut cursor = Cursor::new(data);

        // ── Header ──────────────────────────────────────────────
        let _header_size = cursor.read_u32::<LittleEndian>()?;
        let _unknown1 = cursor.read_u32::<LittleEndian>()?;
        let flags = cursor.read_u32::<LittleEndian>()?;
        let _unknown2 = cursor.read_u32::<LittleEndian>()?;

        // String table offsets and level name
        let level_name_offset = cursor.read_u32::<LittleEndian>()?;
        let author_offset = cursor.read_u32::<LittleEndian>()?;
        let birth_offset = cursor.read_u32::<LittleEndian>()?;
        let rez_file_offset = cursor.read_u32::<LittleEndian>()?;
        let image_dir_offset = cursor.read_u32::<LittleEndian>()?;
        let pal_rez_offset = cursor.read_u32::<LittleEndian>()?;

        let start_x = cursor.read_i32::<LittleEndian>()?;
        let start_y = cursor.read_i32::<LittleEndian>()?;
        let _unknown3 = cursor.read_i32::<LittleEndian>()?;
        let _unknown4 = cursor.read_i32::<LittleEndian>()?;

        let num_planes = cursor.read_u32::<LittleEndian>()?;
        if num_planes > MAX_PLANES {
            return Err(WwdError::TooManyPlanes(num_planes));
        }

        let exe_file_offset = cursor.read_u32::<LittleEndian>()?;

        // Skip to plane descriptors — remaining header padding
        // We've read 4*16 = 64 bytes so far; header is 1524 bytes total
        let mut _skip = vec![0u8; WWD_HEADER_SIZE - 64];
        cursor.read_exact(&mut _skip)?;

        // ── Read string table entries ───────────────────────────
        let level_name = read_string_table_entry(data, level_name_offset)?;
        let author = read_string_table_entry(data, author_offset)?;
        let birth = read_string_table_entry(data, birth_offset)?;
        let rez_file = read_string_table_entry(data, rez_file_offset)?;
        let image_dir = read_string_table_entry(data, image_dir_offset)?;
        let pal_rez = read_string_table_entry(data, pal_rez_offset)?;
        let exe_file = read_string_table_entry(data, exe_file_offset)?;

        let header = WwdHeader {
            flags,
            level_name,
            author,
            birth,
            rez_file,
            image_dir,
            pal_rez,
            start_x,
            start_y,
            num_planes,
            exe_file,
        };

        // ── Plane descriptors ───────────────────────────────────
        let plane_desc_start = WWD_HEADER_SIZE;
        let mut planes = Vec::with_capacity(num_planes as usize);

        for i in 0..num_planes as usize {
            let desc_offset = plane_desc_start + i * PLANE_DESCRIPTOR_SIZE;
            if desc_offset + PLANE_DESCRIPTOR_SIZE > data.len() {
                return Err(WwdError::PlaneOutOfBounds(i));
            }

            let mut pc = Cursor::new(&data[desc_offset..]);

            let _plane_size = pc.read_u32::<LittleEndian>()?;
            let _unknown = pc.read_u32::<LittleEndian>()?;
            let plane_flags = pc.read_u32::<LittleEndian>()?;

            let plane_name_offset = pc.read_u32::<LittleEndian>()?;

            let tile_width = pc.read_u32::<LittleEndian>()?;
            let tile_height = pc.read_u32::<LittleEndian>()?;

            let movement_x_percent = pc.read_u32::<LittleEndian>()?;
            let movement_y_percent = pc.read_u32::<LittleEndian>()?;

            let fill_color = pc.read_u32::<LittleEndian>()?;

            let _num_image_sets = pc.read_u32::<LittleEndian>()?;
            let _num_objects = pc.read_u32::<LittleEndian>()?;

            let tile_data_offset = pc.read_u32::<LittleEndian>()? as usize;
            let _image_sets_offset = pc.read_u32::<LittleEndian>()? as usize;
            let _objects_offset = pc.read_u32::<LittleEndian>()? as usize;

            let z_coord = pc.read_i32::<LittleEndian>()?;

            let tiles_wide = pc.read_u32::<LittleEndian>()?;
            let tiles_high = pc.read_u32::<LittleEndian>()?;

            let plane_name = read_string_table_entry(data, plane_name_offset)?;

            // ── Read tile data ──────────────────────────────────
            let tile_count = (tiles_wide as usize)
                .checked_mul(tiles_high as usize)
                .ok_or_else(|| WwdError::TooManyTiles {
                    plane: plane_name.clone(),
                    count: usize::MAX,
                })?;
            if tile_count > MAX_TILES_PER_PLANE {
                return Err(WwdError::TooManyTiles {
                    plane: plane_name.clone(),
                    count: tile_count,
                });
            }
            let tile_data_end = tile_data_offset + tile_count * 4;
            if tile_data_end > data.len() {
                return Err(WwdError::TileDataOutOfBounds(plane_name.clone()));
            }

            let mut tiles = Vec::with_capacity(tile_count);
            let mut tc = Cursor::new(&data[tile_data_offset..]);
            for _ in 0..tile_count {
                tiles.push(tc.read_u32::<LittleEndian>()?);
            }

            // ── Read image sets ─────────────────────────────────
            let mut image_sets = Vec::new();
            if _image_sets_offset > 0 && _image_sets_offset < data.len() {
                for _ in 0.._num_image_sets {
                    // Image set entries are stored as sequential null-terminated
                    // strings in the string table area. We read them via the
                    // image sets offset which points to an array of string
                    // offsets (u32 LE each).
                    // Simplified: read from the offset table
                }
                // Alternative approach: read string offsets
                let offsets_end = _image_sets_offset + (_num_image_sets as usize) * 4;
                if offsets_end <= data.len() {
                    let mut isc = Cursor::new(&data[_image_sets_offset..]);
                    for _ in 0.._num_image_sets {
                        let str_off = isc.read_u32::<LittleEndian>()?;
                        if str_off > 0 {
                            let s = read_string_table_entry(data, str_off)?;
                            image_sets.push(s);
                        }
                    }
                }
            }

            // ── Read objects ────────────────────────────────────
            if _num_objects > MAX_OBJECTS_PER_PLANE {
                return Err(WwdError::TooManyObjects {
                    plane: plane_name.clone(),
                    count: _num_objects,
                });
            }
            let mut objects = Vec::new();
            if _objects_offset > 0 && _objects_offset < data.len() {
                let mut oc = Cursor::new(&data[_objects_offset..]);
                for _ in 0.._num_objects {
                    match Self::parse_object(&mut oc, data) {
                        Ok(obj) => objects.push(obj),
                        Err(_) => break,
                    }
                }
            }

            planes.push(WwdPlane {
                name: plane_name,
                flags: plane_flags,
                tile_width,
                tile_height,
                movement_x_percent,
                movement_y_percent,
                fill_color,
                z_coord,
                tiles_wide,
                tiles_high,
                tiles,
                image_sets,
                objects,
            });
        }

        Ok(WwdFile { header, planes })
    }

    /// Parse a single object from the object data stream.
    fn parse_object(cursor: &mut Cursor<&[u8]>, full_data: &[u8]) -> Result<WwdObject, WwdError> {
        let id = cursor.read_u32::<LittleEndian>()?;

        let name_offset = cursor.read_u32::<LittleEndian>()?;
        let logic_offset = cursor.read_u32::<LittleEndian>()?;
        let image_set_offset = cursor.read_u32::<LittleEndian>()?;
        let animation_offset = cursor.read_u32::<LittleEndian>()?;

        let x = cursor.read_i32::<LittleEndian>()?;
        let y = cursor.read_i32::<LittleEndian>()?;
        let z = cursor.read_i32::<LittleEndian>()?;

        let i_value = cursor.read_i32::<LittleEndian>()?;

        let speed_x = cursor.read_i32::<LittleEndian>()?;
        let speed_y = cursor.read_i32::<LittleEndian>()?;

        let damage = cursor.read_i32::<LittleEndian>()?;
        let smarts = cursor.read_i32::<LittleEndian>()?;
        let health = cursor.read_i32::<LittleEndian>()?;

        let score = cursor.read_i32::<LittleEndian>()?;
        let points = cursor.read_i32::<LittleEndian>()?;
        let powerup = cursor.read_i32::<LittleEndian>()?;
        let direction = cursor.read_i32::<LittleEndian>()?;

        let draw_flags = cursor.read_u32::<LittleEndian>()?;
        let user_flags = cursor.read_u32::<LittleEndian>()?;

        let hit_rect = WwdRect::read_from(cursor)?;
        let attack_rect = WwdRect::read_from(cursor)?;
        let clip_rect = WwdRect::read_from(cursor)?;

        // User rects (2)
        let user_rect1 = WwdRect::read_from(cursor)?;
        let user_rect2 = WwdRect::read_from(cursor)?;
        let user_rects = vec![user_rect1, user_rect2];

        // User values (8)
        let mut user_values = Vec::with_capacity(8);
        for _ in 0..8 {
            user_values.push(cursor.read_i32::<LittleEndian>()?);
        }

        let move_rect = WwdRect::read_from(cursor)?;

        // Resolve string offsets
        let name = read_string_table_entry(full_data, name_offset).unwrap_or_default();
        let logic = read_string_table_entry(full_data, logic_offset).unwrap_or_default();
        let image_set =
            read_string_table_entry(full_data, image_set_offset).unwrap_or_default();
        let animation =
            read_string_table_entry(full_data, animation_offset).unwrap_or_default();

        Ok(WwdObject {
            id,
            name,
            logic,
            image_set,
            animation,
            x,
            y,
            z,
            i_value,
            speed_x,
            speed_y,
            damage,
            smarts,
            health,
            score,
            points,
            powerup,
            direction,
            hit_rect,
            attack_rect,
            clip_rect,
            user_rects,
            user_values,
            move_rect,
            draw_flags,
            user_flags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid WWD with 0 planes for header parsing test.
    fn make_minimal_wwd() -> Vec<u8> {
        let string_table_offset: u32 = (WWD_HEADER_SIZE) as u32;

        // String table: all strings are empty (just null bytes)
        let string_table = b"\0\0\0\0\0\0\0";

        let mut buf = vec![0u8; WWD_HEADER_SIZE];
        let mut cursor = Cursor::new(&mut buf[..]);

        use byteorder::WriteBytesExt;

        // header_size
        cursor.write_u32::<LittleEndian>(WWD_HEADER_SIZE as u32).unwrap();
        // unknown1
        cursor.write_u32::<LittleEndian>(0).unwrap();
        // flags
        cursor.write_u32::<LittleEndian>(0).unwrap();
        // unknown2
        cursor.write_u32::<LittleEndian>(0).unwrap();

        // String offsets — all point to string_table_offset
        for _ in 0..6 {
            cursor.write_u32::<LittleEndian>(string_table_offset).unwrap();
        }

        // start_x, start_y
        cursor.write_i32::<LittleEndian>(100).unwrap();
        cursor.write_i32::<LittleEndian>(200).unwrap();
        // unknown3, unknown4
        cursor.write_i32::<LittleEndian>(0).unwrap();
        cursor.write_i32::<LittleEndian>(0).unwrap();

        // num_planes = 0
        cursor.write_u32::<LittleEndian>(0).unwrap();
        // exe_file offset
        cursor.write_u32::<LittleEndian>(string_table_offset).unwrap();

        // Rest is already zeroed

        buf.extend_from_slice(string_table);
        buf
    }

    #[test]
    fn test_parse_minimal_header() {
        let data = make_minimal_wwd();
        let wwd = WwdFile::parse(&data).unwrap();
        assert_eq!(wwd.header.start_x, 100);
        assert_eq!(wwd.header.start_y, 200);
        assert_eq!(wwd.header.num_planes, 0);
        assert!(wwd.planes.is_empty());
    }

    #[test]
    fn test_data_too_short() {
        let data = vec![0u8; 10];
        assert!(WwdFile::parse(&data).is_err());
    }
}
