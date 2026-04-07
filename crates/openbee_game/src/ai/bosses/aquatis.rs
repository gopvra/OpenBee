//! Aquatis boss AI: water-based attacks and minion spawning.

use glam::Vec2;
use openbee_core::ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

/// Aquatis boss phases based on health thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AquatisPhase {
    /// Phase 1 (100%-75% health): Basic water attacks.
    WaterSpout,
    /// Phase 2 (75%-50% health): Spawns fish minions.
    MinionSwarm,
    /// Phase 3 (50%-25% health): Whirlpool attack.
    Whirlpool,
    /// Phase 4 (<25% health): All attacks combined, enraged.
    Enraged,
}

/// Attack patterns for Aquatis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AquatisAttack {
    /// Shoots a stream of water at the player.
    WaterJet,
    /// Creates a rising water column from below.
    Geyser,
    /// Spawns fish minions that home in on the player.
    SpawnMinions,
    /// Creates a whirlpool that pulls the player in.
    WhirlpoolSuck,
    /// Slams tentacles down at the player's position.
    TentacleSlam,
    /// Dives underwater and resurfaces elsewhere.
    Dive,
}

/// AI controller for the Aquatis boss encounter.
#[derive(Debug, Clone)]
pub struct AquatisBossAi {
    /// Current phase based on health.
    pub phase: AquatisPhase,
    /// Current attack being executed.
    pub current_attack: Option<AquatisAttack>,
    /// Timer for the current attack animation/execution.
    pub attack_timer: f32,
    /// Cooldown between attacks.
    pub attack_cooldown: f32,
    /// Cooldown timer.
    pub cooldown_timer: f32,
    /// Number of minions currently alive.
    pub minion_count: u32,
    /// Maximum minions allowed at once.
    pub max_minions: u32,
    /// Whether the boss is currently submerged.
    pub submerged: bool,
    /// Home position (center of arena).
    pub arena_center: Vec2,
    /// Target player entity.
    pub target: Option<Entity>,
    /// Health threshold for phase 2 (fraction 0-1).
    pub phase2_threshold: f32,
    /// Health threshold for phase 3.
    pub phase3_threshold: f32,
    /// Health threshold for phase 4.
    pub phase4_threshold: f32,
}

impl AquatisBossAi {
    /// Create a new Aquatis AI at the given arena center.
    pub fn new(arena_center: Vec2) -> Self {
        Self {
            phase: AquatisPhase::WaterSpout,
            current_attack: None,
            attack_timer: 0.0,
            attack_cooldown: 2.0,
            cooldown_timer: 0.0,
            minion_count: 0,
            max_minions: 4,
            submerged: false,
            arena_center,
            target: None,
            phase2_threshold: 0.75,
            phase3_threshold: 0.50,
            phase4_threshold: 0.25,
        }
    }

    /// Update phase based on current health fraction.
    pub fn update_phase(&mut self, health_fraction: f32) {
        self.phase = if health_fraction <= self.phase4_threshold {
            AquatisPhase::Enraged
        } else if health_fraction <= self.phase3_threshold {
            AquatisPhase::Whirlpool
        } else if health_fraction <= self.phase2_threshold {
            AquatisPhase::MinionSwarm
        } else {
            AquatisPhase::WaterSpout
        };
    }

    /// Choose the next attack based on the current phase.
    pub fn choose_attack(&mut self) -> AquatisAttack {
        let attack = match self.phase {
            AquatisPhase::WaterSpout => {
                if rand::random::<f32>() < 0.6 {
                    AquatisAttack::WaterJet
                } else {
                    AquatisAttack::Geyser
                }
            }
            AquatisPhase::MinionSwarm => {
                if self.minion_count < self.max_minions && rand::random::<f32>() < 0.4 {
                    AquatisAttack::SpawnMinions
                } else {
                    AquatisAttack::WaterJet
                }
            }
            AquatisPhase::Whirlpool => {
                let r = rand::random::<f32>();
                if r < 0.3 {
                    AquatisAttack::WhirlpoolSuck
                } else if r < 0.6 {
                    AquatisAttack::TentacleSlam
                } else {
                    AquatisAttack::Geyser
                }
            }
            AquatisPhase::Enraged => {
                let r = rand::random::<f32>();
                if r < 0.2 {
                    AquatisAttack::Dive
                } else if r < 0.4 {
                    AquatisAttack::WhirlpoolSuck
                } else if r < 0.6 {
                    AquatisAttack::TentacleSlam
                } else if r < 0.8 && self.minion_count < self.max_minions {
                    AquatisAttack::SpawnMinions
                } else {
                    AquatisAttack::WaterJet
                }
            }
        };
        self.current_attack = Some(attack);
        self.attack_timer = 0.0;
        attack
    }

    /// Update the boss AI each frame.
    pub fn update(&mut self, dt: f32, health_fraction: f32) {
        self.update_phase(health_fraction);

        if self.current_attack.is_some() {
            self.attack_timer += dt;
            // Attack duration depends on type
            let duration = match self.current_attack.unwrap() {
                AquatisAttack::WaterJet => 1.5,
                AquatisAttack::Geyser => 1.0,
                AquatisAttack::SpawnMinions => 0.8,
                AquatisAttack::WhirlpoolSuck => 3.0,
                AquatisAttack::TentacleSlam => 1.2,
                AquatisAttack::Dive => 2.5,
            };
            if self.attack_timer >= duration {
                self.current_attack = None;
                self.cooldown_timer = self.attack_cooldown;
            }
        } else {
            self.cooldown_timer -= dt;
            if self.cooldown_timer <= 0.0 {
                self.choose_attack();
            }
        }
    }

    /// Get the attack cooldown modifier for the current phase.
    pub fn cooldown_modifier(&self) -> f32 {
        match self.phase {
            AquatisPhase::WaterSpout => 1.0,
            AquatisPhase::MinionSwarm => 0.9,
            AquatisPhase::Whirlpool => 0.75,
            AquatisPhase::Enraged => 0.5,
        }
    }
}

impl Component for AquatisBossAi {}
