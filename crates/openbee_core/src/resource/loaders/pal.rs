//! Loader for Captain Claw PAL (palette) files.

use super::ResourceLoader;
use anyhow::{bail, Result};
use std::any::Any;

/// A 256-color palette.
#[derive(Debug, Clone)]
pub struct PalResource {
    /// 256 RGB entries (768 bytes total).
    pub colors: Vec<[u8; 3]>,
}

/// Loader for `.pal` palette files.
#[derive(Clone)]
pub struct PalLoader;

impl ResourceLoader for PalLoader {
    fn extensions(&self) -> &[&str] {
        &["pal"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        if data.len() < 768 {
            bail!(
                "PAL file too small: {} ({} bytes, expected >= 768)",
                path,
                data.len()
            );
        }

        let colors: Vec<[u8; 3]> = data[..768]
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();

        tracing::debug!("PalLoader: loaded {} ({} colors)", path, colors.len());
        Ok(Box::new(PalResource { colors }))
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
