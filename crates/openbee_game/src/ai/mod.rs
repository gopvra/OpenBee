//! AI behaviors for enemies, bosses, and special objects.

pub mod bosses;
pub mod crumbling_peg;
pub mod enemy_ai;
pub mod projectile_ai;
pub mod punk_rat;
pub mod state_machine;
pub mod toggle_peg;

pub use enemy_ai::{AiState, EnemyAiComponent, EnemyType};
pub use state_machine::StateMachine;
