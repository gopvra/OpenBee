//! Red Tail boss AI: acrobatic, fast attacks.

use glam::Vec2;
use openbee_core::ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

/// Red Tail's combat phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedTailPhase {
    /// Phase 1 (100%-70%): Quick strikes and flips.
    Acrobatic,
    /// Phase 2 (70%-40%): Adds bomb throws and wall jumps.
    Explosive,
    /// Phase 3 (<40%): Full fury, all attacks faster.
    Fury,
}

/// Attack patterns for Red Tail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedTailAttack {
    /// Quick forward dash slash.
    DashSlash,
    /// Backflip kick (evasive attack).
    BackflipKick,
    /// Wall jump into aerial dive attack.
    WallDive,
    /// Throws a bomb at the player.
    BombThrow,
    /// Quick jab combo (1-2 hits).
    QuickJab,
    /// Smoke bomb - disappears and reappears behind player.
    SmokeBomb,
    /// Aerial cartwheel attack.
    Cartwheel,
    /// Rapid multi-hit flurry.
    Flurry,
}

/// AI controller for the Red Tail boss encounter.
#[derive(Debug, Clone)]
pub struct RedTailBossAi {
    pub phase: RedTailPhase,
    pub current_attack: Option<RedTailAttack>,
    pub attack_timer: f32,
    pub cooldown_timer: f32,
    pub arena_center: Vec2,
    pub target: Option<Entity>,
    /// Whether Red Tail is currently airborne (from jumps/flips).
    pub airborne: bool,
    /// Consecutive attacks without retreating.
    pub aggression_counter: u32,
    /// Maximum attacks before forced retreat.
    pub max_aggression: u32,
    /// Whether currently invisible (smoke bomb).
    pub invisible: bool,
    pub invisible_timer: f32,
    pub phase2_threshold: f32,
    pub phase3_threshold: f32,
}

impl RedTailBossAi {
    pub fn new(arena_center: Vec2) -> Self {
        Self {
            phase: RedTailPhase::Acrobatic,
            current_attack: None,
            attack_timer: 0.0,
            cooldown_timer: 0.8,
            arena_center,
            target: None,
            airborne: false,
            aggression_counter: 0,
            max_aggression: 4,
            invisible: false,
            invisible_timer: 0.0,
            phase2_threshold: 0.70,
            phase3_threshold: 0.40,
        }
    }

    pub fn update_phase(&mut self, health_fraction: f32) {
        self.phase = if health_fraction <= self.phase3_threshold {
            RedTailPhase::Fury
        } else if health_fraction <= self.phase2_threshold {
            RedTailPhase::Explosive
        } else {
            RedTailPhase::Acrobatic
        };
    }

    pub fn choose_attack(&mut self) -> RedTailAttack {
        // If too aggressive, force a retreat move
        if self.aggression_counter >= self.max_aggression {
            self.aggression_counter = 0;
            let attack = if rand::random::<f32>() < 0.5 {
                RedTailAttack::BackflipKick
            } else {
                RedTailAttack::SmokeBomb
            };
            self.current_attack = Some(attack);
            self.attack_timer = 0.0;
            return attack;
        }

        let attack = match self.phase {
            RedTailPhase::Acrobatic => {
                let r = rand::random::<f32>();
                if r < 0.3 {
                    RedTailAttack::DashSlash
                } else if r < 0.5 {
                    RedTailAttack::BackflipKick
                } else if r < 0.75 {
                    RedTailAttack::QuickJab
                } else {
                    RedTailAttack::Cartwheel
                }
            }
            RedTailPhase::Explosive => {
                let r = rand::random::<f32>();
                if r < 0.2 {
                    RedTailAttack::BombThrow
                } else if r < 0.4 {
                    RedTailAttack::WallDive
                } else if r < 0.6 {
                    RedTailAttack::DashSlash
                } else if r < 0.8 {
                    RedTailAttack::SmokeBomb
                } else {
                    RedTailAttack::QuickJab
                }
            }
            RedTailPhase::Fury => {
                let r = rand::random::<f32>();
                if r < 0.2 {
                    RedTailAttack::Flurry
                } else if r < 0.35 {
                    RedTailAttack::WallDive
                } else if r < 0.5 {
                    RedTailAttack::BombThrow
                } else if r < 0.65 {
                    RedTailAttack::SmokeBomb
                } else if r < 0.8 {
                    RedTailAttack::DashSlash
                } else {
                    RedTailAttack::Cartwheel
                }
            }
        };
        self.current_attack = Some(attack);
        self.attack_timer = 0.0;
        self.aggression_counter += 1;
        attack
    }

    pub fn update(&mut self, dt: f32, health_fraction: f32) {
        self.update_phase(health_fraction);

        // Update invisibility
        if self.invisible {
            self.invisible_timer -= dt;
            if self.invisible_timer <= 0.0 {
                self.invisible = false;
            }
        }

        if self.current_attack.is_some() {
            self.attack_timer += dt;
            let duration = match self.current_attack.unwrap() {
                RedTailAttack::DashSlash => 0.5,
                RedTailAttack::BackflipKick => 0.7,
                RedTailAttack::WallDive => 1.2,
                RedTailAttack::BombThrow => 0.6,
                RedTailAttack::QuickJab => 0.4,
                RedTailAttack::SmokeBomb => 0.3,
                RedTailAttack::Cartwheel => 0.8,
                RedTailAttack::Flurry => 1.5,
            };
            if self.attack_timer >= duration {
                if self.current_attack == Some(RedTailAttack::SmokeBomb) {
                    self.invisible = true;
                    self.invisible_timer = 1.5;
                }
                self.current_attack = None;
                self.cooldown_timer = match self.phase {
                    RedTailPhase::Acrobatic => 1.0,
                    RedTailPhase::Explosive => 0.6,
                    RedTailPhase::Fury => 0.3,
                };
            }
        } else {
            self.cooldown_timer -= dt;
            if self.cooldown_timer <= 0.0 {
                self.choose_attack();
            }
        }
    }

    pub fn is_vulnerable(&self) -> bool {
        !self.invisible
    }
}

impl Component for RedTailBossAi {}
