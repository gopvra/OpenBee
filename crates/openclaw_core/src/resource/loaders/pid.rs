//! Loader for Captain Claw PID (image) files.

use std::any::Any;
use anyhow::{Result, bail};
use super::ResourceLoader;

/// Parsed PID image data.
#[derive(Debug, Clone)]
pub struct PidResource {
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    /// RGBA pixel data.
    pub pixels: Vec<u8>,
}

/// Loader for `.pid` image files used in Captain Claw.
#[derive(Clone)]
pub struct PidLoader;

impl ResourceLoader for PidLoader {
    fn extensions(&self) -> &[&str] {
        &["pid"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        if data.len() < 8 {
            bail!("PID file too small: {} ({} bytes)", path, data.len());
        }
        tracing::debug!("PidLoader: loaded {} ({} bytes)", path, data.len());
        Ok(Box::new(PidResource {
            width: 0,
            height: 0,
            offset_x: 0,
            offset_y: 0,
            pixels: Vec::new(),
        }))
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
