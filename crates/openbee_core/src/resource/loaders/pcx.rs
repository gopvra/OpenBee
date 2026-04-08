//! Loader for PCX image files.

use super::ResourceLoader;
use anyhow::{bail, Result};
use std::any::Any;

/// Parsed PCX image data.
#[derive(Debug, Clone)]
pub struct PcxResource {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data.
    pub pixels: Vec<u8>,
}

/// Loader for `.pcx` image files.
#[derive(Clone)]
pub struct PcxLoader;

impl ResourceLoader for PcxLoader {
    fn extensions(&self) -> &[&str] {
        &["pcx"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        if data.len() < 128 {
            bail!("PCX file too small: {} ({} bytes)", path, data.len());
        }
        // PCX header parsing would go here; actual implementation is in openbee_rez.
        tracing::debug!("PcxLoader: loaded {} ({} bytes)", path, data.len());
        Ok(Box::new(PcxResource {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }))
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
