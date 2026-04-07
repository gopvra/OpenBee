//! Trigger system: evaluates area-based triggers.

use openbee_core::ecs::{System, World};

use crate::components::score::ScoreComponent;
use crate::components::transform::TransformComponent;
use crate::components::triggers::sound_trigger::SoundTriggerComponent;
use crate::components::triggers::trigger::{TriggerComponent, TriggerType};

/// Checks all trigger regions for player overlap and fires events.
pub struct TriggerSystem;

impl System for TriggerSystem {
    fn name(&self) -> &str {
        "TriggerSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<_> = world.entities();

        // Find player position
        let mut player_pos = None;
        for &entity in &entities {
            if world.has_component::<ScoreComponent>(entity) {
                if let Some(transform) = world.get_component::<TransformComponent>(entity) {
                    player_pos = Some(transform.position);
                    break;
                }
            }
        }

        let player_pos = match player_pos {
            Some(p) => p,
            None => return,
        };

        // Check triggers
        for &entity in &entities {
            let should_trigger = {
                let trigger = match world.get_component::<TriggerComponent>(entity) {
                    Some(t) => t,
                    None => continue,
                };

                if trigger.one_shot && trigger.triggered {
                    continue;
                }

                trigger.trigger_rect.contains(player_pos)
            };

            if should_trigger {
                if let Some(trigger) = world.get_component_mut::<TriggerComponent>(entity) {
                    trigger.triggered = true;

                    match trigger.trigger_type {
                        TriggerType::BossFight => {
                            tracing::info!("Boss fight trigger activated");
                        }
                        TriggerType::Sound => {
                            // Check for associated sound trigger component
                        }
                        TriggerType::LevelEnd => {
                            tracing::info!("Level end trigger activated");
                        }
                        TriggerType::Teleport => {
                            tracing::info!("Teleport trigger activated");
                        }
                        _ => {}
                    }
                }
            }

            // Sound triggers
            let should_play_sound = {
                let st = match world.get_component::<SoundTriggerComponent>(entity) {
                    Some(s) => s,
                    None => continue,
                };
                !st.played
            };

            if should_play_sound {
                if let Some(transform) = world.get_component::<TransformComponent>(entity) {
                    let dist = (player_pos - transform.position).length();
                    if dist < 100.0 {
                        if let Some(st) = world.get_component_mut::<SoundTriggerComponent>(entity) {
                            st.played = true;
                            // Sound playback delegated to audio system
                        }
                    }
                }
            }
        }
    }
}
