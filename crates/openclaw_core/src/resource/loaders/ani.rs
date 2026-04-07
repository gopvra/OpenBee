//! Loader for Captain Claw ANI (animation) files.

use std::any::Any;
use anyhow::{Result, bail};
use super::ResourceLoader;

/// Parsed ANI animation data.
#[derive(Debug, Clone)]
pub struct AniResource {
    /// Number of frames in the animation.
    pub frame_count: u32,
    /// Raw frame data (format-specific).
    pub raw_frames: Vec<Vec<u8>>,
}

/// Loader for `.ani` animation files.
#[derive(Clone)]
pub struct AniLoader;

impl ResourceLoader for AniLoader {
    fn extensions(&self) -> &[&str] {
        &["ani"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        if data.len() < 4 {
            bail!("ANI file too small: {} ({})", path, data.len());
        }
        // Placeholder: actual parsing is in openclaw_rez.
        // Return a minimal resource so the loader pipeline works.
        tracing::debug!("AniLoader: loaded {} ({} bytes)", path, data.len());
        Ok(Box::new(AniResource {
            frame_count: 0,
            raw_frames: Vec::new(),
        }))
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
