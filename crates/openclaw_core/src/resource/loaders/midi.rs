//! Loader for MIDI music files using the `midly` crate.

use std::any::Any;
use anyhow::Result;
use super::ResourceLoader;
use crate::audio::music::MidiData;

/// Loader for `.mid` / `.midi` / `.xmi` music files.
#[derive(Clone)]
pub struct MidiLoader;

impl ResourceLoader for MidiLoader {
    fn extensions(&self) -> &[&str] {
        &["mid", "midi", "xmi"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        // Validate that midly can parse the header.
        let _smf = midly::Smf::parse(data)?;

        tracing::debug!("MidiLoader: loaded {} ({} bytes)", path, data.len());
        Ok(Box::new(MidiData::new(data.to_vec())))
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
