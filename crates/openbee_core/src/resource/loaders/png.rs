//! Loader for PNG image files using the `image` crate.

use super::ResourceLoader;
use anyhow::Result;
use std::any::Any;

/// Parsed PNG image data.
#[derive(Debug, Clone)]
pub struct PngResource {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data.
    pub pixels: Vec<u8>,
}

/// Loader for `.png` image files.
#[derive(Clone)]
pub struct PngLoader;

impl ResourceLoader for PngLoader {
    fn extensions(&self) -> &[&str] {
        &["png"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        let img = image::load_from_memory_with_format(data, image::ImageFormat::Png)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        tracing::debug!("PngLoader: loaded {} ({}x{})", path, width, height);
        Ok(Box::new(PngResource {
            width,
            height,
            pixels: rgba.into_raw(),
        }))
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
