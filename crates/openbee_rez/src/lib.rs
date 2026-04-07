//! `openbee_rez` - CLAW.REZ archive and asset format parser for OpenBee.
//!
//! This crate provides parsers for the various proprietary file formats used
//! by the 1997 game Captain Claw, including:
//!
//! - **REZ archive** (`CLAW.REZ`) - the main game archive containing all assets
//! - **WWD** - level/world data format
//! - **ANI** - animation descriptors
//! - **PID** - palette-indexed images
//! - **PCX** - standard PCX images (used for palettes/backgrounds)
//! - **PAL** - 256-color palettes

pub mod rez_archive;
pub mod wwd_parser;
pub mod ani_parser;
pub mod pid_parser;
pub mod pcx_parser;
pub mod pal_parser;

pub use rez_archive::{RezArchive, RezEntry};
pub use wwd_parser::{WwdFile, WwdHeader, WwdPlane, WwdObject, WwdRect};
pub use ani_parser::{AniFile, AniFrame};
pub use pid_parser::PidImage;
pub use pcx_parser::PcxImage;
pub use pal_parser::Palette;
