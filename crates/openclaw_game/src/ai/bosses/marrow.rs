//! Captain Marrow boss AI: sword combat and dodge mechanics.

use glam::Vec2;
use openclaw_core::ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

/// Captain Marrow's combat phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarrowPhase {
    /// Phase 1 (100%-65%): Basic sword attacks and parrying.
    Dueling,
    /// Phase 2 (65%-35%): Adds dodge rolls and combo attacks.
    Aggressive,
    /// Phase 3 (<35%): Enraged, faster attacks, pistol shots.
    Enraged,
}

/// Attack patterns for Captain Marrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarrowAttack {
    /// Basic horizontal sword slash.
    Slash,
    /// Overhead sword slam.
    Overhead,
    /// Three-hit sword combo.
    Combo,
    /// Dodge roll to evade player attacks.
    DodgeRoll,
    /// Block incoming attacks.
    Parry,
    /// Fire a pistol shot.
    PistolShot,
    /// Lunging thrust attack.
    Lunge,
    /// Spin attack hitting in all directions.
    SpinAttack,
}

/// AI controller for the Captain Marrow boss encounter.
#[derive(Debug, Clone)]
pub struct MarrowBossAi {
    pub phase: MarrowPhase,
    pub current_attack: Option<MarrowAttack>,
    pub attack_timer: f32,
    pub cooldown_timer: f32,
    pub combo_count: u32,
    pub max_combo: u32,
    pub is_blocking: bool,
    pub block_timer: f32,
    pub dodge_cooldown: f32,
    pub arena_center: Vec2,
    pub target: Option<Entity>,
    pub phase2_threshold: f32,
    pub phase3_threshold: f32,
    /// Number of hits taken without retaliating (triggers aggressive response).
    pub hits_taken: u32,
}

impl MarrowBossAi {
    pub fn new(arena_center: Vec2) -> Self {
        Self {
            phase: MarrowPhase::Dueling,
            current_attack: None,
            attack_timer: 0.0,
            cooldown_timer: 1.0,
            combo_count: 0,
            max_combo: 3,
            is_blocking: false,
            block_timer: 0.0,
            dodge_cooldown: 0.0,
            arena_center,
            target: None,
            phase2_threshold: 0.65,
            phase3_threshold: 0.35,
            hits_taken: 0,
        }
    }

    pub fn update_phase(&mut self, health_fraction: f32) {
        self.phase = if health_fraction <= self.phase3_threshold {
            MarrowPhase::Enraged
        } else if health_fraction <= self.phase2_threshold {
            MarrowPhase::Aggressive
        } else {
            MarrowPhase::Dueling
        };
    }

    pub fn choose_attack(&mut self) -> MarrowAttack {
        let attack = match self.phase {
            MarrowPhase::Dueling => {
                let r = rand::random::<f32>();
                if r < 0.4 {
                    MarrowAttack::Slash
                } else if r < 0.7 {
                    MarrowAttack::Overhead
                } else {
                    MarrowAttack::Parry
                }
            }
            MarrowPhase::Aggressive => {
                let r = rand::random::<f32>();
                if r < 0.25 {
                    MarrowAttack::Combo
                } else if r < 0.45 && self.dodge_cooldown <= 0.0 {
                    MarrowAttack::DodgeRoll
                } else if r < 0.65 {
                    MarrowAttack::Lunge
                } else if r < 0.85 {
                    MarrowAttack::Slash
                } else {
                    MarrowAttack::Overhead
                }
            }
            MarrowPhase::Enraged => {
                let r = rand::random::<f32>();
                if r < 0.2 {
                    MarrowAttack::SpinAttack
                } else if r < 0.4 {
                    MarrowAttack::Combo
                } else if r < 0.55 {
                    MarrowAttack::PistolShot
                } else if r < 0.7 {
                    MarrowAttack::Lunge
                } else if r < 0.85 && self.dodge_cooldown <= 0.0 {
                    MarrowAttack::DodgeRoll
                } else {
                    MarrowAttack::Slash
                }
            }
        };
        self.current_attack = Some(attack);
        self.attack_timer = 0.0;
        self.combo_count = 0;
        attack
    }

    pub fn update(&mut self, dt: f32, health_fraction: f32) {
        self.update_phase(health_fraction);
        self.dodge_cooldown = (self.dodge_cooldown - dt).max(0.0);

        if self.is_blocking {
            self.block_timer -= dt;
            if self.block_timer <= 0.0 {
                self.is_blocking = false;
            }
        }

        if self.current_attack.is_some() {
            self.attack_timer += dt;
            let duration = match self.current_attack.unwrap() {
                MarrowAttack::Slash => 0.6,
                MarrowAttack::Overhead => 0.8,
                MarrowAttack::Combo => 1.5,
                MarrowAttack::DodgeRoll => 0.5,
                MarrowAttack::Parry => 0.3,
                MarrowAttack::PistolShot => 0.7,
                MarrowAttack::Lunge => 0.9,
                MarrowAttack::SpinAttack => 1.2,
            };
            if self.attack_timer >= duration {
                if self.current_attack == Some(MarrowAttack::Parry) {
                    self.is_blocking = true;
                    self.block_timer = 1.0;
                }
                if self.current_attack == Some(MarrowAttack::DodgeRoll) {
                    self.dodge_cooldown = 3.0;
                }
                self.current_attack = None;
                self.cooldown_timer = match self.phase {
                    MarrowPhase::Dueling => 1.5,
                    MarrowPhase::Aggressive => 0.8,
                    MarrowPhase::Enraged => 0.4,
                };
            }
        } else {
            self.cooldown_timer -= dt;
            if self.cooldown_timer <= 0.0 {
                self.choose_attack();
            }
        }
    }

    /// Called when the boss is hit by the player.
    pub fn on_hit(&mut self) {
        self.hits_taken += 1;
        // After 3 unretliated hits, force a counterattack
        if self.hits_taken >= 3 && self.current_attack.is_none() {
            self.choose_attack();
            self.hits_taken = 0;
        }
    }

    pub fn is_vulnerable(&self) -> bool {
        !self.is_blocking
    }
}

impl Component for MarrowBossAi {}
