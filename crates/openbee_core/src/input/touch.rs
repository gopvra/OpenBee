//! Touch input types for mobile/touch screen support.

/// Unique identifier for a touch point (finger).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TouchId(pub u64);

/// State of a single touch point.
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    /// Unique identifier of this touch.
    pub id: TouchId,
    /// Horizontal position in screen pixels.
    pub x: f32,
    /// Vertical position in screen pixels.
    pub y: f32,
    /// Pressure (0.0..=1.0) if supported, otherwise 1.0.
    pub pressure: f32,
}

impl TouchPoint {
    /// Create a new touch point.
    pub fn new(id: TouchId, x: f32, y: f32, pressure: f32) -> Self {
        Self { id, x, y, pressure }
    }
}

/// Recognized touch gestures.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TouchGesture {
    /// A quick tap at the given position.
    Tap { x: f32, y: f32 },
    /// A swipe from start to end position.
    Swipe {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
    },
    /// A two-finger pinch gesture.
    Pinch {
        /// Center of the pinch.
        center_x: f32,
        center_y: f32,
        /// Scale factor (>1.0 = spread, <1.0 = pinch).
        scale: f32,
    },
}
