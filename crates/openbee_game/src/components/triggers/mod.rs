//! Trigger components for event-driven gameplay.

pub mod boss_stager;
pub mod sound_trigger;
pub mod trigger;

pub use boss_stager::{BossStagerComponent, BossType};
pub use sound_trigger::SoundTriggerComponent;
pub use trigger::{TriggerComponent, TriggerType};
