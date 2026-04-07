//! AI system: runs enemy AI state machines each frame.

use glam::Vec2;
use openbee_core::ecs::{System, World};

use crate::ai::enemy_ai::{AiState, EnemyAiComponent};
use crate::components::kinematic::KinematicComponent;
use crate::components::transform::TransformComponent;

/// Runs all enemy AI behaviors, updating states and issuing movement commands.
pub struct AiSystem;

impl System for AiSystem {
    fn name(&self) -> &str {
        "AiSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        // First pass: find player position for targeting.
        // In a full implementation this would query for the player entity.
        // For now we iterate and process each AI independently.
        let player_position: Option<Vec2> = None; // Set by game logic externally

        for entity in entities {
            let should_update = world
                .get_component::<EnemyAiComponent>(entity)
                .map_or(false, |ai| {
                    ai.state != AiState::Dead
                });

            if !should_update {
                continue;
            }

            // Get current position
            let my_position = world
                .get_component::<TransformComponent>(entity)
                .map(|t| t.position)
                .unwrap_or(Vec2::ZERO);

            // Update think timer and state
            if let Some(ai) = world.get_component_mut::<EnemyAiComponent>(entity) {
                ai.state_timer += dt;
                ai.think_timer += dt;

                let think_interval = ai.think_interval();
                if ai.think_timer < think_interval {
                    continue;
                }
                ai.think_timer = 0.0;

                match ai.state {
                    AiState::Idle => {
                        // Transition to patrol after idle time
                        if ai.state_timer > 2.0 {
                            ai.transition(AiState::Patrolling);
                        }
                        // Check for player in detection range
                        if let Some(player_pos) = player_position {
                            let dist = (player_pos - my_position).length();
                            if dist < ai.detection_range {
                                ai.transition(AiState::Chasing);
                            }
                        }
                    }
                    AiState::Patrolling => {
                        // Check for player
                        if let Some(player_pos) = player_position {
                            let dist = (player_pos - my_position).length();
                            if dist < ai.detection_range {
                                ai.transition(AiState::Chasing);
                            }
                        }
                        // Patrol movement is handled below
                    }
                    AiState::Chasing => {
                        if let Some(player_pos) = player_position {
                            let dist = (player_pos - my_position).length();
                            if dist < ai.attack_range {
                                ai.transition(AiState::Attacking);
                            } else if dist > ai.detection_range * 1.5 {
                                ai.transition(AiState::Retreating);
                            }
                        } else {
                            ai.transition(AiState::Idle);
                        }
                    }
                    AiState::Attacking => {
                        // Return to chasing after attack completes
                        if ai.state_timer > 1.0 {
                            ai.transition(AiState::Chasing);
                        }
                    }
                    AiState::Retreating => {
                        let dist_home = (ai.home_position - my_position).length();
                        if dist_home < 10.0 {
                            ai.transition(AiState::Idle);
                        }
                    }
                    AiState::Stunned => {
                        if ai.state_timer > 1.5 {
                            ai.transition(AiState::Idle);
                        }
                    }
                    AiState::Dying => {
                        if ai.state_timer > 1.0 {
                            ai.transition(AiState::Dead);
                        }
                    }
                    AiState::Dead => {}
                }

                // Apply movement based on state
                let home = ai.home_position;
                let patrol_range = ai.patrol_range;
                let facing = ai.facing;
                let state = ai.state;

                if let Some(kinematic) = world.get_component_mut::<KinematicComponent>(entity) {
                    match state {
                        AiState::Patrolling => {
                            kinematic.velocity.x = facing * 80.0;
                            // Reverse at patrol bounds
                            let dist_home = (my_position.x - home.x).abs();
                            if dist_home > patrol_range {
                                if let Some(ai) = world.get_component_mut::<EnemyAiComponent>(entity) {
                                    ai.facing = -ai.facing;
                                }
                            }
                        }
                        AiState::Chasing => {
                            if let Some(player_pos) = player_position {
                                let dir = (player_pos.x - my_position.x).signum();
                                kinematic.velocity.x = dir * 150.0;
                                if let Some(ai) = world.get_component_mut::<EnemyAiComponent>(entity) {
                                    ai.facing = dir;
                                }
                            }
                        }
                        AiState::Retreating => {
                            let dir = (home.x - my_position.x).signum();
                            kinematic.velocity.x = dir * 100.0;
                        }
                        AiState::Attacking | AiState::Stunned | AiState::Dying | AiState::Dead => {
                            kinematic.velocity.x = 0.0;
                        }
                        AiState::Idle => {
                            kinematic.velocity.x = 0.0;
                        }
                    }
                }
            }
        }
    }
}
