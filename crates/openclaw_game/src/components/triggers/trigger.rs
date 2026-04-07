//! Generic trigger component for area-based event activation.

use openclaw_core::ecs::{Component, Entity};
use openclaw_core::render::Rect;
/// Type of event a trigger can fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerType {
    /// Starts a boss fight sequence.
    BossFight,
    /// Plays a sound effect.
    Sound,
    /// Displays a message or dialogue.
    Dialogue,
    /// Activates or deactivates another entity.
    Toggle,
    /// Teleports the player to a position.
    Teleport,
    /// Starts a scripted cutscene.
    Cutscene,
    /// Opens a door or gate.
    DoorOpen,
    /// Level transition.
    LevelEnd,
}

/// An area-based trigger that fires events when the player enters its bounds.
#[derive(Debug, Clone)]
pub struct TriggerComponent {
    /// Type of trigger event.
    pub trigger_type: TriggerType,
    /// Bounding rectangle in world coordinates.
    pub trigger_rect: Rect,
    /// Whether the trigger fires only once.
    pub one_shot: bool,
    /// Whether the trigger has already been activated.
    pub triggered: bool,
    /// Optional entity affected by the trigger.
    pub target_entity: Option<Entity>,
}

impl Default for TriggerComponent {
    fn default() -> Self {
        Self {
            trigger_type: TriggerType::Toggle,
            trigger_rect: Rect::new(0.0, 0.0, 64.0, 64.0),
            one_shot: false,
            triggered: false,
            target_entity: None,
        }
    }
}

impl Component for TriggerComponent {}
