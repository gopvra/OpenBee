//! Score system: manages score calculations and extra life thresholds.

use openbee_core::ecs::{System, World};

use crate::components::life::LifeComponent;
use crate::components::score::ScoreComponent;

/// Points threshold for earning an extra life.
const EXTRA_LIFE_THRESHOLD: u64 = 100_000;

/// Monitors score milestones and awards extra lives.
pub struct ScoreSystem;

impl System for ScoreSystem {
    fn name(&self) -> &str {
        "ScoreSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<_> = world.entities();

        for &entity in &entities {
            if let (Some(score), Some(_life)) = (
                world.get_component::<ScoreComponent>(entity),
                world.get_component::<LifeComponent>(entity),
            ) {
                // Check if score crossed an extra life threshold
                let prev_lives_earned =
                    ((score.score.saturating_sub(1)) / EXTRA_LIFE_THRESHOLD) as i32;
                let _ = prev_lives_earned; // Would compare with a stored counter
            }
        }
    }
}
