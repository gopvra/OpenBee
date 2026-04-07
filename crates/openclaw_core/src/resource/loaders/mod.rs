//! Resource loader trait and format-specific loaders.

pub mod ani;
pub mod midi;
pub mod pal;
pub mod pcx;
pub mod pid;
pub mod png;
pub mod wav;
pub mod wwd;
pub mod xml;

use std::any::Any;

use anyhow::Result;

/// Trait for resource loaders that can parse raw bytes into typed resource objects.
pub trait ResourceLoader: Send + Sync {
    /// Return the file extensions this loader handles (lowercase, no dot).
    fn extensions(&self) -> &[&str];

    /// Load a resource from raw bytes. The path is provided for error messages.
    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>>;

    /// Clone this loader into a boxed trait object (for registration across multiple extensions).
    fn clone_box(&self) -> Box<dyn ResourceLoader>;
}
