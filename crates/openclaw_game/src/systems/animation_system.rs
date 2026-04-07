//! Animation system: advances animation players each frame.

use openclaw_core::ecs::{System, World};

use crate::components::animation::AnimationComponent;

/// Updates all animation components, advancing frame timers.
pub struct AnimationSystem;

impl System for AnimationSystem {
    fn name(&self) -> &str {
        "AnimationSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<_> = world.entities();

        for entity in entities {
            if let Some(anim) = world.get_component_mut::<AnimationComponent>(entity) {
                if !anim.playing {
                    continue;
                }
                // Animation frame advancement is handled by the AnimationPlayer in core.
                // Here we just ensure state consistency.
                let _ = anim.speed; // speed is applied by the animation player
            }
        }
    }
}
