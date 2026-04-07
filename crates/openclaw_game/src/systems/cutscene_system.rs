//! Cutscene system: scripted sequences with camera, actor, and dialogue actions.

use openclaw_core::ecs::{System, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single action in a cutscene script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CutsceneAction {
    // Camera
    /// Pan the camera to a position over a duration.
    CameraPanTo { x: f32, y: f32, duration: f32 },
    /// Zoom the camera to a level over a duration.
    CameraZoom { zoom: f32, duration: f32 },
    /// Shake the camera with a given intensity for a duration.
    CameraShake { intensity: f32, duration: f32 },

    // Actors
    /// Move an actor entity to a position at a given speed.
    MoveActor { entity_id: u64, x: f32, y: f32, speed: f32 },
    /// Play a named animation on an actor entity.
    PlayAnimation { entity_id: u64, animation: String },
    /// Spawn a new actor from a template at a position.
    SpawnActor { template: String, x: f32, y: f32 },
    /// Destroy an actor entity.
    DestroyActor { entity_id: u64 },

    // Visual
    /// Fade in from black over a duration.
    FadeIn { duration: f32 },
    /// Fade out to a color over a duration.
    FadeOut { duration: f32, color: [u8; 4] },
    /// Show an image overlay at a position for a duration.
    ShowImage { path: String, x: f32, y: f32, duration: f32 },
    /// Set letterbox bars (0.0 = none, 1.0 = full).
    SetLetterbox { amount: f32 },

    // Audio
    /// Play a sound effect.
    PlaySound { path: String },
    /// Play background music.
    PlayMusic { path: String },
    /// Stop music with a fade-out duration.
    StopMusic { fade_duration: f32 },

    // Dialogue
    /// Show a dialogue box with speaker name, text, and optional portrait.
    ShowDialogue { speaker: String, text: String, portrait: Option<String> },
    /// Show a multiple-choice prompt.
    ShowChoice { choices: Vec<String> },

    // Flow
    /// Wait for a number of seconds.
    Wait { seconds: f32 },
    /// Set a named game state.
    SetGameState { state: String },
    /// Disable player input.
    DisablePlayerControl,
    /// Enable player input.
    EnablePlayerControl,
    /// Set a named boolean flag.
    SetFlag { name: String, value: bool },
    /// Jump to a step index if a flag is true.
    ConditionalJump { flag: String, step: usize },

    // Screen effects
    /// Flash the screen with a color for a duration.
    ScreenFlash { color: [u8; 4], duration: f32 },
    /// Slow-motion effect with a time scale for a duration.
    SlowMotion { time_scale: f32, duration: f32 },
}

/// A complete cutscene script consisting of ordered actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutsceneScript {
    /// Name identifier for this cutscene.
    pub name: String,
    /// Ordered list of actions.
    pub actions: Vec<CutsceneAction>,
    /// Whether the player can skip this cutscene.
    pub skippable: bool,
}

/// Plays back a cutscene script, stepping through actions frame by frame.
pub struct CutscenePlayer {
    /// The currently loaded script.
    pub script: Option<CutsceneScript>,
    /// Index of the current step in the script.
    pub current_step: usize,
    /// Whether playback is in progress.
    pub is_playing: bool,
    /// Whether we are waiting (e.g. for a Wait action timer).
    pub is_waiting: bool,
    /// Countdown timer for Wait actions.
    pub wait_timer: f32,
    /// Flags that can be set and checked during the cutscene.
    pub flags: HashMap<String, bool>,
    /// The player's chosen option when a ShowChoice was presented.
    pub chosen_option: Option<usize>,
}

impl CutscenePlayer {
    /// Create a new idle cutscene player.
    pub fn new() -> Self {
        Self {
            script: None,
            current_step: 0,
            is_playing: false,
            is_waiting: false,
            wait_timer: 0.0,
            flags: HashMap::new(),
            chosen_option: None,
        }
    }

    /// Start playing a cutscene script from the beginning.
    pub fn play(&mut self, script: CutsceneScript) {
        self.script = Some(script);
        self.current_step = 0;
        self.is_playing = true;
        self.is_waiting = false;
        self.wait_timer = 0.0;
        self.chosen_option = None;
    }

    /// Stop the current cutscene immediately.
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_waiting = false;
        self.wait_timer = 0.0;
        self.script = None;
        self.current_step = 0;
        self.chosen_option = None;
    }

    /// Skip to the end of the current cutscene if it is skippable.
    pub fn skip(&mut self) {
        if let Some(ref script) = self.script {
            if script.skippable {
                self.stop();
            }
        }
    }

    /// Advance to the next step in the script.
    pub fn advance(&mut self) {
        if let Some(ref script) = self.script {
            self.current_step += 1;
            if self.current_step >= script.actions.len() {
                self.is_playing = false;
            }
        }
        self.is_waiting = false;
        self.wait_timer = 0.0;
        self.chosen_option = None;
    }

    /// Update timers and return the current action to process, if any.
    pub fn update(&mut self, dt: f32) -> Option<CutsceneAction> {
        if !self.is_playing {
            return None;
        }

        let script = match self.script {
            Some(ref s) => s,
            None => return None,
        };

        if self.current_step >= script.actions.len() {
            self.is_playing = false;
            return None;
        }

        let actions_len = script.actions.len();

        // If we are waiting on a timer, tick it down.
        if self.is_waiting {
            self.wait_timer -= dt;
            if self.wait_timer <= 0.0 {
                self.is_waiting = false;
                self.wait_timer = 0.0;
                self.advance();
                // After advancing, check if there's a new action.
                if !self.is_playing {
                    return None;
                }
                if self.current_step >= actions_len {
                    self.is_playing = false;
                    return None;
                }
            } else {
                return None;
            }
        }

        // Re-borrow script since advance may have changed state but not the Option.
        let script = self.script.as_ref()?;
        if self.current_step >= script.actions.len() {
            self.is_playing = false;
            return None;
        }

        let action = script.actions[self.current_step].clone();

        // Handle flow control actions internally.
        match &action {
            CutsceneAction::Wait { seconds } => {
                self.is_waiting = true;
                self.wait_timer = *seconds;
                return Some(action);
            }
            CutsceneAction::SetFlag { name, value } => {
                self.flags.insert(name.clone(), *value);
                self.advance();
                // Recurse to get the next real action.
                return self.update(0.0);
            }
            CutsceneAction::ConditionalJump { flag, step } => {
                let flag_val = self.flags.get(flag).copied().unwrap_or(false);
                if flag_val {
                    self.current_step = *step;
                } else {
                    self.advance();
                }
                return self.update(0.0);
            }
            CutsceneAction::ShowChoice { .. } => {
                // Wait for player to make a choice.
                if self.chosen_option.is_none() {
                    return Some(action);
                }
                // Choice was made, advance.
                self.advance();
                return self.update(0.0);
            }
            _ => {
                // For all other actions, return them and advance.
                self.advance();
                return Some(action);
            }
        }
    }

    /// Whether a cutscene is currently active.
    pub fn is_active(&self) -> bool {
        self.is_playing
    }

    /// Set the player's choice for a ShowChoice action.
    pub fn set_choice(&mut self, index: usize) {
        self.chosen_option = Some(index);
    }
}

impl Default for CutscenePlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// System that drives cutscene playback each frame.
pub struct CutsceneSystem {
    /// The cutscene player that manages script state.
    pub player: CutscenePlayer,
}

impl CutsceneSystem {
    /// Create a new cutscene system with an idle player.
    pub fn new() -> Self {
        Self {
            player: CutscenePlayer::new(),
        }
    }
}

impl Default for CutsceneSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for CutsceneSystem {
    fn name(&self) -> &str {
        "CutsceneSystem"
    }

    fn update(&mut self, _world: &mut World, dt: f32) {
        // Process the current cutscene step.
        if let Some(action) = self.player.update(dt) {
            // In a full implementation, each action would interact with the World
            // (move camera, spawn entities, play audio, etc.). For now, we log the
            // action type for debugging.
            match &action {
                CutsceneAction::DisablePlayerControl => {
                    tracing::debug!("Cutscene: disabling player control");
                }
                CutsceneAction::EnablePlayerControl => {
                    tracing::debug!("Cutscene: enabling player control");
                }
                CutsceneAction::FadeIn { duration } => {
                    tracing::debug!("Cutscene: fade in over {duration}s");
                }
                CutsceneAction::FadeOut { duration, .. } => {
                    tracing::debug!("Cutscene: fade out over {duration}s");
                }
                CutsceneAction::CameraPanTo { x, y, duration } => {
                    tracing::debug!("Cutscene: pan camera to ({x},{y}) over {duration}s");
                }
                CutsceneAction::ShowDialogue { speaker, text, .. } => {
                    tracing::debug!("Cutscene: {speaker}: {text}");
                }
                _ => {
                    tracing::trace!("Cutscene: processing action");
                }
            }
        }
    }
}
