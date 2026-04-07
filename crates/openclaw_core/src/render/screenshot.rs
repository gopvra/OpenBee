//! Screenshot capture and saving system.

use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

/// Manages screenshot capture requests and writes pixel data to image files.
pub struct ScreenshotManager {
    /// Directory where screenshots are saved.
    pub output_directory: PathBuf,
    /// Image format to use when saving.
    pub format: ScreenshotFormat,
    /// Whether to automatically generate filenames with incrementing counter.
    pub auto_name: bool,
    /// Running counter for auto-named screenshots.
    pub counter: u32,
    /// Flag set to `true` when a capture has been requested but not yet fulfilled.
    pub pending_capture: bool,
    /// Path of the most recently saved screenshot, if any.
    pub last_screenshot_path: Option<PathBuf>,
}

/// Supported image formats for screenshot output.
#[derive(Debug, Clone, Copy)]
pub enum ScreenshotFormat {
    Png,
    Jpg,
    Bmp,
}

impl ScreenshotFormat {
    /// File extension string for the format (without leading dot).
    fn extension(&self) -> &'static str {
        match self {
            ScreenshotFormat::Png => "png",
            ScreenshotFormat::Jpg => "jpg",
            ScreenshotFormat::Bmp => "bmp",
        }
    }
}

impl ScreenshotManager {
    /// Create a new manager that saves screenshots to `output_dir`.
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_directory: output_dir,
            format: ScreenshotFormat::Png,
            auto_name: true,
            counter: 0,
            pending_capture: false,
            last_screenshot_path: None,
        }
    }

    /// Mark that a screenshot should be captured on the next frame.
    pub fn request_capture(&mut self) {
        self.pending_capture = true;
    }

    /// Capture a screenshot from raw RGBA pixel data and save it to disk.
    ///
    /// `pixels` must contain exactly `width * height * 4` bytes in RGBA order.
    /// Returns the path the image was saved to.
    pub fn capture(&mut self, pixels: &[u8], width: u32, height: u32) -> Result<PathBuf> {
        let expected_len = (width as usize) * (height as usize) * 4;
        anyhow::ensure!(
            pixels.len() == expected_len,
            "pixel buffer size mismatch: expected {} bytes, got {}",
            expected_len,
            pixels.len()
        );

        let path = self.generate_filename();

        // Ensure output directory exists.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let image_buffer: image::ImageBuffer<image::Rgba<u8>, _> =
            image::ImageBuffer::from_raw(width, height, pixels.to_vec())
                .ok_or_else(|| anyhow::anyhow!("failed to create image buffer from raw pixels"))?;

        match self.format {
            ScreenshotFormat::Png => image_buffer.save_with_format(&path, image::ImageFormat::Png)?,
            ScreenshotFormat::Jpg => {
                image_buffer.save_with_format(&path, image::ImageFormat::Jpeg)?
            }
            ScreenshotFormat::Bmp => image_buffer.save_with_format(&path, image::ImageFormat::Bmp)?,
        }

        info!("Screenshot saved to {}", path.display());

        self.pending_capture = false;
        self.counter += 1;
        self.last_screenshot_path = Some(path.clone());
        Ok(path)
    }

    /// Generate the next filename based on the current counter and format.
    pub fn generate_filename(&self) -> PathBuf {
        let filename = if self.auto_name {
            format!("screenshot_{:04}.{}", self.counter, self.format.extension())
        } else {
            format!("screenshot.{}", self.format.extension())
        };
        self.output_directory.join(filename)
    }

    /// Change the image format used for future screenshots.
    pub fn set_format(&mut self, format: ScreenshotFormat) {
        self.format = format;
    }
}
