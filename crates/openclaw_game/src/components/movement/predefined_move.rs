//! Predefined movement patterns for platforms and obstacles.

use openclaw_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Type of predefined movement pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveType {
    /// Linear back-and-forth movement.
    Linear,
    /// Sinusoidal oscillation.
    Sine,
    /// Circular orbit.
    Circular,
}

/// Axis on which the predefined movement operates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveAxis {
    Horizontal,
    Vertical,
    Both,
}

/// An entity that follows a scripted movement pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredefinedMoveComponent {
    /// Type of movement pattern.
    pub move_type: MoveType,
    /// Speed or frequency of the movement.
    pub speed: f32,
    /// Distance or amplitude in pixels.
    pub distance: f32,
    /// Axis of movement.
    pub axis: MoveAxis,
    /// Internal phase accumulator.
    #[serde(skip)]
    pub phase: f32,
}

impl Default for PredefinedMoveComponent {
    fn default() -> Self {
        Self {
            move_type: MoveType::Linear,
            speed: 100.0,
            distance: 64.0,
            axis: MoveAxis::Horizontal,
            phase: 0.0,
        }
    }
}

impl Component for PredefinedMoveComponent {}
