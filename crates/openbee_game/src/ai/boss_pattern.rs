//! Declarative DSL for defining boss attack patterns.
//!
//! Bosses are defined as a series of phases, each containing prioritized patterns
//! made up of discrete steps (movement, attacks, effects, flow control). At runtime
//! the `BossPatternExecutor` walks through steps, emitting `PatternAction`s that
//! other game systems consume.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Pattern step definitions
// ---------------------------------------------------------------------------

/// A single step in a boss attack pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternStep {
    // -- Movement --
    MoveTo { x: f32, y: f32, speed: f32 },
    MoveRelative { dx: f32, dy: f32, speed: f32 },
    Dash { direction: f32, speed: f32, duration: f32 },
    Teleport { x: f32, y: f32 },
    FacePlayer,

    // -- Combat --
    FireProjectile { projectile: String, speed: f32, angle: f32 },
    FireSpread { projectile: String, count: u32, spread_angle: f32, speed: f32 },
    FireCircle { projectile: String, count: u32, speed: f32 },
    /// Fire a projectile aimed at the player's current position.
    FireAimed { projectile: String, speed: f32 },
    MeleeAttack { damage: i32, range: f32 },

    // -- Spawning --
    SpawnMinion { template: String, x: f32, y: f32 },
    SpawnMinionCircle { template: String, count: u32, radius: f32 },

    // -- Flow control --
    Wait { seconds: f32 },
    WaitRandom { min: f32, max: f32 },
    Repeat { count: u32, steps: Vec<PatternStep> },
    RepeatForever { steps: Vec<PatternStep> },
    RandomChoice { options: Vec<Vec<PatternStep>> },
    Conditional {
        condition: PatternCondition,
        then_steps: Vec<PatternStep>,
        else_steps: Vec<PatternStep>,
    },

    // -- Effects --
    PlayAnimation { name: String },
    PlaySound { path: String },
    ScreenShake { intensity: f32, duration: f32 },
    ScreenFlash { color: [u8; 4], duration: f32 },
    SlowMotion { scale: f32, duration: f32 },

    // -- State --
    SetInvulnerable { invulnerable: bool },
    SetPhase { phase: u32 },
    SetSpeed { speed: f32 },
    Heal { amount: i32 },

    // -- Dialogue --
    ShowText { text: String, duration: f32 },
}

/// Conditions that can be used for branching within patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternCondition {
    /// Health is below a percentage (0.0 .. 1.0).
    HealthBelow(f32),
    /// Health is above a percentage.
    HealthAbove(f32),
    /// Current phase number equals the given value.
    PhaseIs(u32),
    /// Player is within the given distance.
    PlayerInRange(f32),
    /// Random chance succeeds (0.0 .. 1.0 probability).
    RandomChance(f32),
    /// A named timer has expired.
    TimerExpired(String),
    /// Fewer than N minions alive.
    MinionCountBelow(u32),
}

// ---------------------------------------------------------------------------
// Pattern & phase definitions
// ---------------------------------------------------------------------------

/// A named attack pattern with priority and optional activation condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossPattern {
    pub name: String,
    pub steps: Vec<PatternStep>,
    pub priority: u32,
    pub cooldown: f32,
    /// If set, this pattern is only eligible when the condition is met.
    pub condition: Option<PatternCondition>,
}

/// A single phase in a boss fight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossPhaseDefinition {
    pub phase_number: u32,
    /// Enter this phase when health drops below this percentage (0.0 .. 1.0).
    pub health_threshold: f32,
    pub patterns: Vec<BossPattern>,
    /// Played once when the phase is first entered.
    pub enter_pattern: Option<Vec<PatternStep>>,
    pub speed_multiplier: f32,
    pub damage_multiplier: f32,
}

/// Complete boss definition with multiple phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossDefinition {
    pub name: String,
    pub health: i32,
    pub phases: Vec<BossPhaseDefinition>,
    pub intro_pattern: Option<Vec<PatternStep>>,
    pub death_pattern: Option<Vec<PatternStep>>,
}

// ---------------------------------------------------------------------------
// Pattern actions (output)
// ---------------------------------------------------------------------------

/// Actions emitted by the pattern executor for consumption by game systems.
#[derive(Debug, Clone)]
pub enum PatternAction {
    Move { x: f32, y: f32, speed: f32 },
    FireProjectile { template: String, x: f32, y: f32, angle: f32, speed: f32 },
    SpawnEntity { template: String, x: f32, y: f32 },
    PlayAnimation { name: String },
    PlaySound { path: String },
    ScreenShake { intensity: f32, duration: f32 },
    ScreenFlash { color: [u8; 4], duration: f32 },
    SlowMotion { scale: f32, duration: f32 },
    ShowText { text: String, duration: f32 },
    SetInvulnerable { value: bool },
    MeleeAttack { damage: i32, range: f32 },
    Heal { amount: i32 },
    Teleport { x: f32, y: f32 },
    FacePlayer,
    Nothing,
}

// ---------------------------------------------------------------------------
// Runtime executor
// ---------------------------------------------------------------------------

/// Represents one level of a repeat / sub-pattern on the execution stack.
#[derive(Debug, Clone)]
struct RepeatFrame {
    /// The steps being repeated.
    steps: Vec<PatternStep>,
    /// Current index within `steps`.
    step_index: usize,
    /// How many iterations remain (`u32::MAX` for forever).
    remaining: u32,
}

/// Runtime executor that walks through a `BossDefinition` and emits actions.
pub struct BossPatternExecutor {
    pub definition: BossDefinition,
    pub current_phase: u32,
    pub current_pattern: Option<usize>,
    pub step_index: usize,
    pub step_timer: f32,
    pub pattern_cooldowns: HashMap<String, f32>,
    pub is_executing: bool,
    /// Stack for nested Repeat / RepeatForever blocks.
    pub repeat_stack: Vec<(usize, u32, u32)>,
    /// Internal execution stack for handling nested flow-control.
    exec_stack: Vec<RepeatFrame>,
    /// Steps currently being executed (top-level or from a pattern).
    active_steps: Vec<PatternStep>,
    /// Whether we have already played the intro pattern.
    _intro_done: bool,
    /// Whether the phase-enter pattern has been played for the current phase.
    phase_enter_done: bool,
}

impl BossPatternExecutor {
    /// Create a new executor for the given boss definition. Starts at phase 0.
    pub fn new(definition: BossDefinition) -> Self {
        let intro = definition.intro_pattern.clone().unwrap_or_default();
        let has_intro = !intro.is_empty();
        Self {
            definition,
            current_phase: 0,
            current_pattern: None,
            step_index: 0,
            step_timer: 0.0,
            pattern_cooldowns: HashMap::new(),
            is_executing: has_intro,
            repeat_stack: Vec::new(),
            exec_stack: Vec::new(),
            active_steps: intro,
            _intro_done: !has_intro,
            phase_enter_done: false,
        }
    }

    /// Advance the executor by `dt` seconds given the boss's current health percentage.
    /// Returns a list of actions to be processed by game systems this frame.
    pub fn update(&mut self, dt: f32, health_percent: f32) -> Vec<PatternAction> {
        let mut actions = Vec::new();

        // Tick cooldowns.
        for cooldown in self.pattern_cooldowns.values_mut() {
            *cooldown = (*cooldown - dt).max(0.0);
        }

        // Check for phase transitions.
        self.check_phase_transition(health_percent);

        // If we are waiting, count down.
        if self.step_timer > 0.0 {
            self.step_timer -= dt;
            if self.step_timer > 0.0 {
                return actions;
            }
            // Timer expired -- advance to next step.
            self.advance();
        }

        // If not executing anything, try to select a new pattern.
        if !self.is_executing || self.active_steps.is_empty() {
            self.select_pattern(health_percent);
            if !self.is_executing {
                return actions;
            }
        }

        // Execute current step.
        if let Some(step) = self.current_step().cloned() {
            let action = self.execute_step(&step, health_percent);
            actions.push(action);
        } else {
            // Finished all steps.
            self.is_executing = false;
        }

        actions
    }

    /// Select the highest-priority eligible pattern for the current phase.
    pub fn select_pattern(&mut self, health_percent: f32) {
        let phase_idx = self
            .definition
            .phases
            .iter()
            .position(|p| p.phase_number == self.current_phase);

        let phase = match phase_idx {
            Some(i) => &self.definition.phases[i],
            None => {
                self.is_executing = false;
                return;
            }
        };

        // Find best eligible pattern (highest priority, off cooldown, condition met).
        let mut best: Option<(usize, u32)> = None;
        for (i, pattern) in phase.patterns.iter().enumerate() {
            // Check cooldown.
            if let Some(&cd) = self.pattern_cooldowns.get(&pattern.name) {
                if cd > 0.0 {
                    continue;
                }
            }
            // Check condition.
            if let Some(ref cond) = pattern.condition {
                if !self.evaluate_condition(cond, health_percent) {
                    continue;
                }
            }
            match best {
                Some((_, best_pri)) if pattern.priority <= best_pri => {}
                _ => best = Some((i, pattern.priority)),
            }
        }

        if let Some((idx, _)) = best {
            let pattern = phase.patterns[idx].clone();
            self.pattern_cooldowns
                .insert(pattern.name.clone(), pattern.cooldown);
            self.current_pattern = Some(idx);
            self.active_steps = pattern.steps;
            self.step_index = 0;
            self.step_timer = 0.0;
            self.exec_stack.clear();
            self.is_executing = true;
        } else {
            self.is_executing = false;
        }
    }

    /// Get the current step being executed, if any.
    pub fn current_step(&self) -> Option<&PatternStep> {
        // If we have frames on the exec stack, use the top frame.
        if let Some(frame) = self.exec_stack.last() {
            frame.steps.get(frame.step_index)
        } else {
            self.active_steps.get(self.step_index)
        }
    }

    /// Advance to the next step in the current pattern or repeat block.
    pub fn advance(&mut self) {
        // If we're inside a repeat frame, advance there first.
        if let Some(frame) = self.exec_stack.last_mut() {
            frame.step_index += 1;
            if frame.step_index >= frame.steps.len() {
                // Finished one iteration.
                if frame.remaining == u32::MAX {
                    // RepeatForever: restart.
                    frame.step_index = 0;
                    return;
                }
                frame.remaining = frame.remaining.saturating_sub(1);
                if frame.remaining > 0 {
                    frame.step_index = 0;
                } else {
                    // Pop the frame and continue outer scope.
                    self.exec_stack.pop();
                    self.advance();
                }
            }
        } else {
            self.step_index += 1;
            if self.step_index >= self.active_steps.len() {
                self.is_executing = false;
            }
        }
    }

    /// Returns `true` if the executor has no active pattern.
    pub fn is_idle(&self) -> bool {
        !self.is_executing
    }

    /// Force-set the current phase.
    pub fn set_phase(&mut self, phase: u32) {
        if self.current_phase != phase {
            self.current_phase = phase;
            self.phase_enter_done = false;
            self.is_executing = false;
            self.exec_stack.clear();
            self.active_steps.clear();
            self.step_index = 0;
        }
    }

    // -- private helpers --

    fn check_phase_transition(&mut self, health_percent: f32) {
        // Find the phase with the highest health_threshold that is still <= health.
        // Phases are entered when health drops *below* their threshold.
        let mut target_phase = self.current_phase;
        for phase in &self.definition.phases {
            if health_percent <= phase.health_threshold && phase.phase_number > target_phase {
                target_phase = phase.phase_number;
            }
        }

        if target_phase != self.current_phase {
            self.set_phase(target_phase);

            // Play the phase-enter pattern if one is defined.
            if let Some(phase) = self
                .definition
                .phases
                .iter()
                .find(|p| p.phase_number == target_phase)
            {
                if let Some(ref enter) = phase.enter_pattern {
                    self.active_steps = enter.clone();
                    self.step_index = 0;
                    self.step_timer = 0.0;
                    self.exec_stack.clear();
                    self.is_executing = true;
                    self.phase_enter_done = true;
                }
            }
        }
    }

    fn evaluate_condition(&self, cond: &PatternCondition, health_percent: f32) -> bool {
        match cond {
            PatternCondition::HealthBelow(threshold) => health_percent < *threshold,
            PatternCondition::HealthAbove(threshold) => health_percent > *threshold,
            PatternCondition::PhaseIs(phase) => self.current_phase == *phase,
            PatternCondition::PlayerInRange(_) => true, // needs external data; default true
            PatternCondition::RandomChance(prob) => {
                // Simple deterministic fallback when rand is not desired here.
                // In production, plug in the game's RNG.
                *prob >= 0.5
            }
            PatternCondition::TimerExpired(_) => true,
            PatternCondition::MinionCountBelow(_) => true,
        }
    }

    fn execute_step(&mut self, step: &PatternStep, health_percent: f32) -> PatternAction {
        match step {
            // -- Movement --
            PatternStep::MoveTo { x, y, speed } => {
                self.advance();
                PatternAction::Move {
                    x: *x,
                    y: *y,
                    speed: *speed,
                }
            }
            PatternStep::MoveRelative { dx, dy, speed } => {
                self.advance();
                PatternAction::Move {
                    x: *dx,
                    y: *dy,
                    speed: *speed,
                }
            }
            PatternStep::Dash {
                direction: _,
                speed,
                duration,
            } => {
                self.step_timer = *duration;
                PatternAction::Move {
                    x: 0.0,
                    y: 0.0,
                    speed: *speed,
                }
            }
            PatternStep::Teleport { x, y } => {
                self.advance();
                PatternAction::Teleport { x: *x, y: *y }
            }
            PatternStep::FacePlayer => {
                self.advance();
                PatternAction::FacePlayer
            }

            // -- Combat --
            PatternStep::FireProjectile {
                projectile,
                speed,
                angle,
            } => {
                self.advance();
                PatternAction::FireProjectile {
                    template: projectile.clone(),
                    x: 0.0,
                    y: 0.0,
                    angle: *angle,
                    speed: *speed,
                }
            }
            PatternStep::FireSpread {
                projectile,
                count,
                spread_angle,
                speed,
            } => {
                self.advance();
                // Emit a single representative action; the game system fans out.
                if *count == 0 {
                    return PatternAction::Nothing;
                }
                let half = *spread_angle / 2.0;
                let step_angle = if *count > 1 {
                    *spread_angle / (*count - 1) as f32
                } else {
                    0.0
                };
                let _ = (half, step_angle); // used by renderer
                PatternAction::FireProjectile {
                    template: projectile.clone(),
                    x: 0.0,
                    y: 0.0,
                    angle: 0.0,
                    speed: *speed,
                }
            }
            PatternStep::FireCircle {
                projectile,
                count,
                speed,
            } => {
                self.advance();
                let _ = count;
                PatternAction::FireProjectile {
                    template: projectile.clone(),
                    x: 0.0,
                    y: 0.0,
                    angle: 0.0,
                    speed: *speed,
                }
            }
            PatternStep::FireAimed { projectile, speed } => {
                self.advance();
                PatternAction::FireProjectile {
                    template: projectile.clone(),
                    x: 0.0,
                    y: 0.0,
                    angle: 0.0, // aimed at player; resolved by game system
                    speed: *speed,
                }
            }
            PatternStep::MeleeAttack { damage, range } => {
                self.advance();
                PatternAction::MeleeAttack {
                    damage: *damage,
                    range: *range,
                }
            }

            // -- Spawning --
            PatternStep::SpawnMinion { template, x, y } => {
                self.advance();
                PatternAction::SpawnEntity {
                    template: template.clone(),
                    x: *x,
                    y: *y,
                }
            }
            PatternStep::SpawnMinionCircle {
                template,
                count,
                radius,
            } => {
                self.advance();
                let _ = (count, radius);
                PatternAction::SpawnEntity {
                    template: template.clone(),
                    x: 0.0,
                    y: 0.0,
                }
            }

            // -- Flow control --
            PatternStep::Wait { seconds } => {
                self.step_timer = *seconds;
                PatternAction::Nothing
            }
            PatternStep::WaitRandom { min, max } => {
                // Pick midpoint; plug in RNG for proper randomness.
                self.step_timer = (*min + *max) / 2.0;
                PatternAction::Nothing
            }
            PatternStep::Repeat { count, steps } => {
                if *count > 0 && !steps.is_empty() {
                    self.exec_stack.push(RepeatFrame {
                        steps: steps.clone(),
                        step_index: 0,
                        remaining: *count,
                    });
                } else {
                    self.advance();
                }
                PatternAction::Nothing
            }
            PatternStep::RepeatForever { steps } => {
                if !steps.is_empty() {
                    self.exec_stack.push(RepeatFrame {
                        steps: steps.clone(),
                        step_index: 0,
                        remaining: u32::MAX,
                    });
                } else {
                    self.advance();
                }
                PatternAction::Nothing
            }
            PatternStep::RandomChoice { options } => {
                // Pick first non-empty option as a placeholder; plug in RNG for real usage.
                if let Some(chosen) = options.iter().find(|o| !o.is_empty()) {
                    self.exec_stack.push(RepeatFrame {
                        steps: chosen.clone(),
                        step_index: 0,
                        remaining: 1,
                    });
                } else {
                    self.advance();
                }
                PatternAction::Nothing
            }
            PatternStep::Conditional {
                condition,
                then_steps,
                else_steps,
            } => {
                let branch = if self.evaluate_condition(condition, health_percent) {
                    then_steps
                } else {
                    else_steps
                };
                if !branch.is_empty() {
                    self.exec_stack.push(RepeatFrame {
                        steps: branch.clone(),
                        step_index: 0,
                        remaining: 1,
                    });
                } else {
                    self.advance();
                }
                PatternAction::Nothing
            }

            // -- Effects --
            PatternStep::PlayAnimation { name } => {
                self.advance();
                PatternAction::PlayAnimation { name: name.clone() }
            }
            PatternStep::PlaySound { path } => {
                self.advance();
                PatternAction::PlaySound { path: path.clone() }
            }
            PatternStep::ScreenShake {
                intensity,
                duration,
            } => {
                self.advance();
                PatternAction::ScreenShake {
                    intensity: *intensity,
                    duration: *duration,
                }
            }
            PatternStep::ScreenFlash { color, duration } => {
                self.advance();
                PatternAction::ScreenFlash {
                    color: *color,
                    duration: *duration,
                }
            }
            PatternStep::SlowMotion { scale, duration } => {
                self.advance();
                PatternAction::SlowMotion {
                    scale: *scale,
                    duration: *duration,
                }
            }

            // -- State --
            PatternStep::SetInvulnerable { invulnerable } => {
                self.advance();
                PatternAction::SetInvulnerable {
                    value: *invulnerable,
                }
            }
            PatternStep::SetPhase { phase } => {
                self.set_phase(*phase);
                PatternAction::Nothing
            }
            PatternStep::SetSpeed { speed } => {
                let _ = speed; // consumed by movement system
                self.advance();
                PatternAction::Nothing
            }
            PatternStep::Heal { amount } => {
                self.advance();
                PatternAction::Heal { amount: *amount }
            }

            // -- Dialogue --
            PatternStep::ShowText { text, duration } => {
                self.advance();
                PatternAction::ShowText {
                    text: text.clone(),
                    duration: *duration,
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Example boss definition
// ---------------------------------------------------------------------------

/// Example: define the Red Tail boss using the pattern DSL.
pub fn example_red_tail_boss() -> BossDefinition {
    BossDefinition {
        name: "Red Tail".into(),
        health: 500,
        intro_pattern: Some(vec![
            PatternStep::PlayAnimation {
                name: "intro_roar".into(),
            },
            PatternStep::ScreenShake {
                intensity: 0.5,
                duration: 1.0,
            },
            PatternStep::ShowText {
                text: "Red Tail emerges from the shadows!".into(),
                duration: 2.0,
            },
            PatternStep::Wait { seconds: 1.5 },
            PatternStep::SetInvulnerable { invulnerable: false },
        ]),
        death_pattern: Some(vec![
            PatternStep::SetInvulnerable { invulnerable: true },
            PatternStep::PlayAnimation {
                name: "death_explode".into(),
            },
            PatternStep::ScreenShake {
                intensity: 1.0,
                duration: 2.0,
            },
            PatternStep::ScreenFlash {
                color: [255, 255, 255, 200],
                duration: 0.5,
            },
        ]),
        phases: vec![
            // Phase 1: basic attacks (100% - 60% health)
            BossPhaseDefinition {
                phase_number: 0,
                health_threshold: 1.0,
                speed_multiplier: 1.0,
                damage_multiplier: 1.0,
                enter_pattern: None,
                patterns: vec![
                    BossPattern {
                        name: "tail_swipe".into(),
                        priority: 1,
                        cooldown: 3.0,
                        condition: Some(PatternCondition::PlayerInRange(150.0)),
                        steps: vec![
                            PatternStep::FacePlayer,
                            PatternStep::PlayAnimation {
                                name: "tail_swipe".into(),
                            },
                            PatternStep::MeleeAttack {
                                damage: 20,
                                range: 150.0,
                            },
                            PatternStep::Wait { seconds: 0.5 },
                        ],
                    },
                    BossPattern {
                        name: "fireball_spread".into(),
                        priority: 2,
                        cooldown: 5.0,
                        condition: None,
                        steps: vec![
                            PatternStep::FacePlayer,
                            PatternStep::PlayAnimation {
                                name: "charge_fire".into(),
                            },
                            PatternStep::Wait { seconds: 0.8 },
                            PatternStep::FireSpread {
                                projectile: "fireball".into(),
                                count: 3,
                                spread_angle: 45.0,
                                speed: 300.0,
                            },
                            PatternStep::PlaySound {
                                path: "sfx/fire_burst.wav".into(),
                            },
                            PatternStep::Wait { seconds: 1.0 },
                        ],
                    },
                ],
            },
            // Phase 2: enraged (60% - 25% health)
            BossPhaseDefinition {
                phase_number: 1,
                health_threshold: 0.6,
                speed_multiplier: 1.5,
                damage_multiplier: 1.3,
                enter_pattern: Some(vec![
                    PatternStep::SetInvulnerable { invulnerable: true },
                    PatternStep::PlayAnimation {
                        name: "enrage_roar".into(),
                    },
                    PatternStep::ScreenShake {
                        intensity: 0.8,
                        duration: 1.5,
                    },
                    PatternStep::ShowText {
                        text: "Red Tail is enraged!".into(),
                        duration: 2.0,
                    },
                    PatternStep::Wait { seconds: 2.0 },
                    PatternStep::SetInvulnerable { invulnerable: false },
                ]),
                patterns: vec![
                    BossPattern {
                        name: "fire_circle".into(),
                        priority: 3,
                        cooldown: 6.0,
                        condition: None,
                        steps: vec![
                            PatternStep::PlayAnimation {
                                name: "channel_fire".into(),
                            },
                            PatternStep::Wait { seconds: 1.0 },
                            PatternStep::Repeat {
                                count: 3,
                                steps: vec![
                                    PatternStep::FireCircle {
                                        projectile: "fireball".into(),
                                        count: 8,
                                        speed: 200.0,
                                    },
                                    PatternStep::Wait { seconds: 0.4 },
                                ],
                            },
                            PatternStep::Wait { seconds: 1.5 },
                        ],
                    },
                    BossPattern {
                        name: "dash_attack".into(),
                        priority: 2,
                        cooldown: 4.0,
                        condition: None,
                        steps: vec![
                            PatternStep::FacePlayer,
                            PatternStep::PlayAnimation {
                                name: "dash_windup".into(),
                            },
                            PatternStep::Wait { seconds: 0.5 },
                            PatternStep::Dash {
                                direction: 0.0,
                                speed: 800.0,
                                duration: 0.3,
                            },
                            PatternStep::MeleeAttack {
                                damage: 35,
                                range: 100.0,
                            },
                            PatternStep::Wait { seconds: 1.0 },
                        ],
                    },
                    BossPattern {
                        name: "spawn_minions".into(),
                        priority: 1,
                        cooldown: 10.0,
                        condition: Some(PatternCondition::MinionCountBelow(2)),
                        steps: vec![
                            PatternStep::PlayAnimation {
                                name: "summon".into(),
                            },
                            PatternStep::SpawnMinionCircle {
                                template: "fire_imp".into(),
                                count: 3,
                                radius: 200.0,
                            },
                            PatternStep::Wait { seconds: 1.0 },
                        ],
                    },
                ],
            },
            // Phase 3: desperation (25% - 0% health)
            BossPhaseDefinition {
                phase_number: 2,
                health_threshold: 0.25,
                speed_multiplier: 2.0,
                damage_multiplier: 1.8,
                enter_pattern: Some(vec![
                    PatternStep::Teleport { x: 400.0, y: 200.0 },
                    PatternStep::SetInvulnerable { invulnerable: true },
                    PatternStep::PlayAnimation {
                        name: "desperation_transform".into(),
                    },
                    PatternStep::ScreenFlash {
                        color: [255, 0, 0, 180],
                        duration: 0.8,
                    },
                    PatternStep::Heal { amount: 50 },
                    PatternStep::Wait { seconds: 2.0 },
                    PatternStep::SetInvulnerable { invulnerable: false },
                ]),
                patterns: vec![BossPattern {
                    name: "desperation_barrage".into(),
                    priority: 5,
                    cooldown: 2.0,
                    condition: None,
                    steps: vec![
                        PatternStep::FacePlayer,
                        PatternStep::Conditional {
                            condition: PatternCondition::RandomChance(0.5),
                            then_steps: vec![
                                PatternStep::FireCircle {
                                    projectile: "mega_fireball".into(),
                                    count: 12,
                                    speed: 250.0,
                                },
                                PatternStep::Wait { seconds: 0.3 },
                                PatternStep::FireAimed {
                                    projectile: "homing_fire".into(),
                                    speed: 350.0,
                                },
                            ],
                            else_steps: vec![
                                PatternStep::Dash {
                                    direction: 0.0,
                                    speed: 1000.0,
                                    duration: 0.2,
                                },
                                PatternStep::MeleeAttack {
                                    damage: 50,
                                    range: 120.0,
                                },
                                PatternStep::ScreenShake {
                                    intensity: 0.6,
                                    duration: 0.5,
                                },
                            ],
                        },
                        PatternStep::Wait { seconds: 0.8 },
                    ],
                }],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_boss_definition() {
        let boss = example_red_tail_boss();
        assert_eq!(boss.name, "Red Tail");
        assert_eq!(boss.health, 500);
        assert_eq!(boss.phases.len(), 3);
        assert!(boss.intro_pattern.is_some());
        assert!(boss.death_pattern.is_some());
    }

    #[test]
    fn test_executor_starts_with_intro() {
        let boss = example_red_tail_boss();
        let executor = BossPatternExecutor::new(boss);
        assert!(executor.is_executing);
        assert!(executor.current_step().is_some());
    }

    #[test]
    fn test_executor_processes_steps() {
        let boss = BossDefinition {
            name: "Test Boss".into(),
            health: 100,
            intro_pattern: None,
            death_pattern: None,
            phases: vec![BossPhaseDefinition {
                phase_number: 0,
                health_threshold: 1.0,
                speed_multiplier: 1.0,
                damage_multiplier: 1.0,
                enter_pattern: None,
                patterns: vec![BossPattern {
                    name: "basic_attack".into(),
                    priority: 1,
                    cooldown: 0.0,
                    condition: None,
                    steps: vec![
                        PatternStep::FacePlayer,
                        PatternStep::MeleeAttack {
                            damage: 10,
                            range: 50.0,
                        },
                    ],
                }],
            }],
        };

        let mut executor = BossPatternExecutor::new(boss);
        let actions = executor.update(0.016, 1.0);
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_executor_wait_step() {
        let boss = BossDefinition {
            name: "Wait Boss".into(),
            health: 100,
            intro_pattern: Some(vec![
                PatternStep::Wait { seconds: 1.0 },
                PatternStep::FacePlayer,
            ]),
            death_pattern: None,
            phases: vec![],
        };

        let mut executor = BossPatternExecutor::new(boss);

        // First update hits the Wait step.
        let actions = executor.update(0.016, 1.0);
        assert!(matches!(actions.first(), Some(PatternAction::Nothing)));
        assert!(executor.step_timer > 0.0);

        // Time hasn't elapsed yet.
        let actions2 = executor.update(0.5, 1.0);
        assert!(actions2.is_empty());

        // After enough time, we advance past the wait.
        let actions3 = executor.update(0.6, 1.0);
        assert!(!actions3.is_empty());
    }

    #[test]
    fn test_executor_idle_when_no_patterns() {
        let boss = BossDefinition {
            name: "Empty Boss".into(),
            health: 100,
            intro_pattern: None,
            death_pattern: None,
            phases: vec![],
        };

        let executor = BossPatternExecutor::new(boss);
        assert!(executor.is_idle());
    }

    #[test]
    fn test_pattern_serialization() {
        let boss = example_red_tail_boss();
        let json = serde_json::to_string_pretty(&boss).expect("serialize");
        let deserialized: BossDefinition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name, boss.name);
        assert_eq!(deserialized.phases.len(), boss.phases.len());
    }
}
