//! Loader for Captain Claw WWD (World Description) files.

use std::any::Any;
use anyhow::{Result, bail};
use super::ResourceLoader;

/// Parsed WWD level data (high-level structure).
#[derive(Debug, Clone)]
pub struct WwdResource {
    /// Level name.
    pub name: String,
    /// Author.
    pub author: String,
    /// Number of planes/layers.
    pub plane_count: u32,
    /// Raw data for further parsing by openbee_rez.
    pub raw_data: Vec<u8>,
}

/// Loader for `.wwd` level description files.
#[derive(Clone)]
pub struct WwdLoader;

impl ResourceLoader for WwdLoader {
    fn extensions(&self) -> &[&str] {
        &["wwd"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        if data.len() < 32 {
            bail!("WWD file too small: {} ({} bytes)", path, data.len());
        }
        // Actual WWD parsing is in openbee_rez; provide raw data for downstream.
        tracing::debug!("WwdLoader: loaded {} ({} bytes)", path, data.len());
        Ok(Box::new(WwdResource {
            name: String::new(),
            author: String::new(),
            plane_count: 0,
            raw_data: data.to_vec(),
        }))
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
