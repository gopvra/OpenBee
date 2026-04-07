//! All ECS systems for the Captain Claw game loop.

pub mod ai_system;
pub mod animation_system;
pub mod audio_system;
pub mod checkpoint_system;
pub mod combat_system;
pub mod hazard_system;
pub mod input_system;
pub mod movement_system;
pub mod particle_system;
pub mod physics_system;
pub mod pickup_system;
pub mod powerup_system;
pub mod render_system;
pub mod score_system;
pub mod spawner_system;
pub mod trigger_system;

use openclaw_core::ecs::World;

/// Register all game systems in the correct execution order.
pub fn register_all_systems(world: &mut World) {
    let scheduler = world.scheduler_mut();
    scheduler.add_system(input_system::InputSystem::new());
    scheduler.add_system(ai_system::AiSystem);
    scheduler.add_system(movement_system::MovementSystem);
    scheduler.add_system(physics_system::PhysicsSystem::new());
    scheduler.add_system(combat_system::CombatSystem);
    scheduler.add_system(hazard_system::HazardSystem);
    scheduler.add_system(pickup_system::PickupSystem);
    scheduler.add_system(checkpoint_system::CheckpointSystem);
    scheduler.add_system(trigger_system::TriggerSystem);
    scheduler.add_system(spawner_system::SpawnerSystem);
    scheduler.add_system(powerup_system::PowerupSystem);
    scheduler.add_system(score_system::ScoreSystem);
    scheduler.add_system(animation_system::AnimationSystem);
    scheduler.add_system(audio_system::AudioSystem);
    scheduler.add_system(particle_system::ParticleSystem);
    scheduler.add_system(render_system::RenderSystem);
}
