//! Base enemy AI component and behavior.

use glam::Vec2;
use openclaw_core::ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

/// Types of enemies in the game, each with unique behavior patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyType {
    Officer,
    Soldier,
    Rat,
    PunkRat,
    CutThroat,
    RobberThief,
    TownGuard,
    SeaGuard,
    Chameleon,
    BearSailor,
    CrazyHook,
    PegLeg,
    Mercat,
    Siren,
    Fish,
    Crab,
}

/// High-level AI state for enemy behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiState {
    /// Standing still, waiting.
    Idle,
    /// Walking back and forth within patrol range.
    Patrolling,
    /// Actively pursuing the player.
    Chasing,
    /// Executing an attack.
    Attacking,
    /// Moving away from the player.
    Retreating,
    /// Temporarily incapacitated.
    Stunned,
    /// Playing death animation.
    Dying,
    /// Fully dead, awaiting cleanup.
    Dead,
}

/// Core enemy AI component managing state, perception, and targeting.
#[derive(Debug, Clone)]
pub struct EnemyAiComponent {
    /// Type of enemy, determines behavior specifics.
    pub ai_type: EnemyType,
    /// Current AI state.
    pub state: AiState,
    /// How far the enemy will patrol from home (pixels).
    pub patrol_range: f32,
    /// Distance at which the enemy detects the player (pixels).
    pub detection_range: f32,
    /// Distance at which the enemy can attack (pixels).
    pub attack_range: f32,
    /// The position the enemy returns to when not chasing.
    pub home_position: Vec2,
    /// The entity being targeted (usually the player).
    pub target: Option<Entity>,
    /// Time accumulator for AI "thinking" intervals.
    pub think_timer: f32,
    /// Time spent in current state.
    pub state_timer: f32,
    /// Direction the enemy is facing (+1 right, -1 left).
    pub facing: f32,
}

impl EnemyAiComponent {
    /// Create a new enemy AI at the given home position.
    pub fn new(ai_type: EnemyType, home_position: Vec2) -> Self {
        let (patrol, detect, attack) = Self::default_ranges(ai_type);
        Self {
            ai_type,
            state: AiState::Idle,
            patrol_range: patrol,
            detection_range: detect,
            attack_range: attack,
            home_position,
            target: None,
            think_timer: 0.0,
            state_timer: 0.0,
            facing: 1.0,
        }
    }

    /// Default perception ranges for each enemy type.
    fn default_ranges(ai_type: EnemyType) -> (f32, f32, f32) {
        match ai_type {
            EnemyType::Officer => (200.0, 400.0, 50.0),
            EnemyType::Soldier => (250.0, 350.0, 200.0),
            EnemyType::Rat => (100.0, 150.0, 30.0),
            EnemyType::PunkRat => (150.0, 300.0, 40.0),
            EnemyType::CutThroat => (200.0, 350.0, 60.0),
            EnemyType::RobberThief => (180.0, 400.0, 50.0),
            EnemyType::TownGuard => (200.0, 300.0, 60.0),
            EnemyType::SeaGuard => (200.0, 300.0, 60.0),
            EnemyType::Chameleon => (100.0, 500.0, 200.0),
            EnemyType::BearSailor => (150.0, 250.0, 70.0),
            EnemyType::CrazyHook => (200.0, 350.0, 60.0),
            EnemyType::PegLeg => (150.0, 300.0, 60.0),
            EnemyType::Mercat => (200.0, 350.0, 40.0),
            EnemyType::Siren => (100.0, 500.0, 300.0),
            EnemyType::Fish => (100.0, 200.0, 30.0),
            EnemyType::Crab => (80.0, 150.0, 30.0),
        }
    }

    /// Transition to a new AI state, resetting the state timer.
    pub fn transition(&mut self, new_state: AiState) {
        if self.state != new_state {
            self.state = new_state;
            self.state_timer = 0.0;
        }
    }

    /// Think interval for this enemy type (seconds between AI decisions).
    pub fn think_interval(&self) -> f32 {
        match self.ai_type {
            EnemyType::Rat | EnemyType::Fish | EnemyType::Crab => 0.3,
            EnemyType::Chameleon | EnemyType::Siren => 0.5,
            _ => 0.25,
        }
    }
}

impl Component for EnemyAiComponent {}
