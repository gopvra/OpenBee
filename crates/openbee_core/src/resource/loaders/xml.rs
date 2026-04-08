//! Loader for XML configuration files.

use super::ResourceLoader;
use anyhow::Result;
use std::any::Any;

/// Parsed XML data stored as a raw string.
#[derive(Debug, Clone)]
pub struct XmlResource {
    /// The raw XML string content.
    pub content: String,
}

/// Loader for `.xml` configuration/data files.
#[derive(Clone)]
pub struct XmlLoader;

impl ResourceLoader for XmlLoader {
    fn extensions(&self) -> &[&str] {
        &["xml"]
    }

    fn load(&self, data: &[u8], path: &str) -> Result<Box<dyn Any + Send + Sync>> {
        let content = String::from_utf8_lossy(data).into_owned();
        // Validate that it's parseable XML.
        let _: quick_xml::events::Event = quick_xml::reader::Reader::from_str(&content)
            .read_event()
            .map_err(|e| anyhow::anyhow!("Invalid XML in {}: {}", path, e))?;
        tracing::debug!("XmlLoader: loaded {} ({} bytes)", path, data.len());
        Ok(Box::new(XmlResource { content }))
    }

    fn clone_box(&self) -> Box<dyn ResourceLoader> {
        Box::new(self.clone())
    }
}
