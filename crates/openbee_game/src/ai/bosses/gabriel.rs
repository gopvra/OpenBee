//! Gabriel boss AI: flying, magic attacks.

use glam::Vec2;
use openbee_core::ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

/// Gabriel's combat phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GabrielPhase {
    /// Phase 1 (100%-60%): Flying swoops and single magic bolts.
    AerialAssault,
    /// Phase 2 (60%-30%): Magic barrage and teleportation.
    MagicBarrage,
    /// Phase 3 (<30%): Desperate all-out attack.
    Desperate,
}

/// Attack patterns for Gabriel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GabrielAttack {
    /// Swoops down to attack the player.
    DiveBomb,
    /// Fires a single magic bolt.
    MagicBolt,
    /// Fires a spread of magic bolts.
    MagicSpread,
    /// Teleports to a new position.
    Teleport,
    /// Creates a magic shield, briefly invulnerable.
    MagicShield,
    /// Summons lightning strikes at the player's position.
    LightningStrike,
    /// Hovers and rains magic projectiles.
    MagicRain,
}

/// AI controller for the Gabriel boss encounter.
#[derive(Debug, Clone)]
pub struct GabrielBossAi {
    pub phase: GabrielPhase,
    pub current_attack: Option<GabrielAttack>,
    pub attack_timer: f32,
    pub cooldown_timer: f32,
    pub fly_height: f32,
    pub fly_target: Vec2,
    pub hovering: bool,
    pub shield_active: bool,
    pub shield_timer: f32,
    pub teleport_cooldown: f32,
    pub arena_center: Vec2,
    pub target: Option<Entity>,
    pub phase2_threshold: f32,
    pub phase3_threshold: f32,
}

impl GabrielBossAi {
    pub fn new(arena_center: Vec2) -> Self {
        Self {
            phase: GabrielPhase::AerialAssault,
            current_attack: None,
            attack_timer: 0.0,
            cooldown_timer: 1.5,
            fly_height: 200.0,
            fly_target: arena_center,
            hovering: true,
            shield_active: false,
            shield_timer: 0.0,
            teleport_cooldown: 0.0,
            arena_center,
            target: None,
            phase2_threshold: 0.60,
            phase3_threshold: 0.30,
        }
    }

    pub fn update_phase(&mut self, health_fraction: f32) {
        self.phase = if health_fraction <= self.phase3_threshold {
            GabrielPhase::Desperate
        } else if health_fraction <= self.phase2_threshold {
            GabrielPhase::MagicBarrage
        } else {
            GabrielPhase::AerialAssault
        };
    }

    pub fn choose_attack(&mut self) -> GabrielAttack {
        let attack = match self.phase {
            GabrielPhase::AerialAssault => {
                if rand::random::<f32>() < 0.5 {
                    GabrielAttack::DiveBomb
                } else {
                    GabrielAttack::MagicBolt
                }
            }
            GabrielPhase::MagicBarrage => {
                let r = rand::random::<f32>();
                if r < 0.25 {
                    GabrielAttack::MagicSpread
                } else if r < 0.5 {
                    GabrielAttack::Teleport
                } else if r < 0.75 {
                    GabrielAttack::LightningStrike
                } else {
                    GabrielAttack::MagicBolt
                }
            }
            GabrielPhase::Desperate => {
                let r = rand::random::<f32>();
                if r < 0.2 {
                    GabrielAttack::MagicRain
                } else if r < 0.4 {
                    GabrielAttack::MagicShield
                } else if r < 0.6 {
                    GabrielAttack::LightningStrike
                } else if r < 0.8 {
                    GabrielAttack::MagicSpread
                } else {
                    GabrielAttack::DiveBomb
                }
            }
        };
        self.current_attack = Some(attack);
        self.attack_timer = 0.0;
        attack
    }

    pub fn update(&mut self, dt: f32, health_fraction: f32) {
        self.update_phase(health_fraction);

        // Update shield
        if self.shield_active {
            self.shield_timer -= dt;
            if self.shield_timer <= 0.0 {
                self.shield_active = false;
            }
        }

        self.teleport_cooldown = (self.teleport_cooldown - dt).max(0.0);

        if let Some(current_attack) = self.current_attack {
            self.attack_timer += dt;
            let duration = match current_attack {
                GabrielAttack::DiveBomb => 1.8,
                GabrielAttack::MagicBolt => 0.6,
                GabrielAttack::MagicSpread => 1.0,
                GabrielAttack::Teleport => 0.8,
                GabrielAttack::MagicShield => 0.5,
                GabrielAttack::LightningStrike => 1.5,
                GabrielAttack::MagicRain => 3.0,
            };
            if self.attack_timer >= duration {
                // Post-attack effects
                if self.current_attack == Some(GabrielAttack::MagicShield) {
                    self.shield_active = true;
                    self.shield_timer = 3.0;
                }
                if self.current_attack == Some(GabrielAttack::Teleport) {
                    self.teleport_cooldown = 5.0;
                }
                self.current_attack = None;
                self.cooldown_timer = match self.phase {
                    GabrielPhase::AerialAssault => 2.0,
                    GabrielPhase::MagicBarrage => 1.2,
                    GabrielPhase::Desperate => 0.6,
                };
            }
        } else {
            self.cooldown_timer -= dt;
            if self.cooldown_timer <= 0.0 {
                self.choose_attack();
            }
        }
    }

    /// Whether Gabriel can be damaged (not shielded).
    pub fn is_vulnerable(&self) -> bool {
        !self.shield_active
    }
}

impl Component for GabrielBossAi {}
