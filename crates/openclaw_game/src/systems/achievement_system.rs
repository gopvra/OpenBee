//! Achievement tracking system for Captain Claw.
//!
//! Tracks player progress, unlocks achievements, queues UI notifications,
//! and persists achievement state to disk.

use openclaw_core::ecs::{System, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defines a single achievement that the player can unlock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementDefinition {
    /// Unique identifier, e.g. `"first_blood"`.
    pub id: String,
    /// Display name shown in the UI.
    pub name: String,
    /// Longer description of how to unlock the achievement.
    pub description: String,
    /// Optional icon asset path.
    pub icon: Option<String>,
    /// If `true` the achievement is hidden until unlocked.
    pub hidden: bool,
    /// Category used for grouping in the achievement menu.
    pub category: AchievementCategory,
    /// Rule that determines when the achievement unlocks.
    pub criteria: AchievementCriteria,
    /// Gamerscore-style point value.
    pub points: u32,
}

/// Achievement categories for grouping in the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AchievementCategory {
    Progression,
    Combat,
    Exploration,
    Speedrun,
    Collection,
    Secret,
    Challenge,
}

/// Describes the condition required to unlock an achievement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCriteria {
    /// Complete N levels.
    CompleteLevels(u32),
    /// Defeat N enemies total.
    DefeatEnemies(u32),
    /// Defeat a specific boss by name.
    DefeatBoss(String),
    /// Collect N treasures total.
    CollectTreasures(u32),
    /// Collect every treasure in a specific level.
    CollectAllInLevel(u32),
    /// Complete the game without dying.
    CompleteWithoutDying,
    /// Complete a level under a time limit (seconds).
    CompleteUnderTime { level: u32, seconds: u32 },
    /// Find a specific named secret.
    FindSecret(String),
    /// Reach a cumulative score.
    ReachScore(u64),
    /// Use a named ability N times.
    UseAbility { ability: String, count: u32 },
    /// Custom / modded achievement criteria.
    Custom(String),
}

/// Tracks how far the player is toward unlocking a specific achievement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementProgress {
    /// `true` once the achievement has been unlocked.
    pub unlocked: bool,
    /// ISO-8601 timestamp of when the achievement was unlocked.
    pub unlock_time: Option<String>,
    /// Normalised progress in `0.0..=1.0`.
    pub progress: f32,
    /// Current raw counter value.
    pub current_value: u32,
    /// Target value required for unlock.
    pub target_value: u32,
    /// Whether the unlock notification has been shown to the player.
    pub notified: bool,
}

impl AchievementProgress {
    /// Create a fresh, locked progress entry with the given target.
    fn new(target: u32) -> Self {
        Self {
            unlocked: false,
            unlock_time: None,
            progress: 0.0,
            current_value: 0,
            target_value: target,
            notified: false,
        }
    }
}

/// The main achievement system.
pub struct AchievementSystem {
    /// All achievement definitions.
    pub definitions: Vec<AchievementDefinition>,
    /// Progress keyed by achievement `id`.
    pub progress: HashMap<String, AchievementProgress>,
    /// Sum of points from unlocked achievements.
    pub total_points: u32,
    /// Queue of achievement IDs whose unlock popup has not yet been shown.
    pub notification_queue: Vec<String>,
    /// How long (seconds) each notification popup stays on screen.
    pub notification_display_time: f32,
    /// Timer counting down for the current notification.
    pub notification_timer: f32,
}

impl AchievementSystem {
    /// Create a new system with the default Captain Claw achievement set.
    pub fn new() -> Self {
        let defs = Self::default_definitions();
        let mut progress = HashMap::new();
        for def in &defs {
            let target = Self::target_for_criteria(&def.criteria);
            progress.insert(def.id.clone(), AchievementProgress::new(target));
        }
        Self {
            definitions: defs,
            progress,
            total_points: 0,
            notification_queue: Vec::new(),
            notification_display_time: 5.0,
            notification_timer: 0.0,
        }
    }

    /// Replace all definitions and reset progress.
    pub fn load_definitions(&mut self, definitions: Vec<AchievementDefinition>) {
        self.progress.clear();
        for def in &definitions {
            let target = Self::target_for_criteria(&def.criteria);
            self.progress
                .insert(def.id.clone(), AchievementProgress::new(target));
        }
        self.definitions = definitions;
        self.total_points = 0;
        self.notification_queue.clear();
    }

    /// Return the full set of default Captain Claw achievements (30+).
    pub fn default_definitions() -> Vec<AchievementDefinition> {
        vec![
            // --- Combat ---
            AchievementDefinition {
                id: "first_blood".into(),
                name: "First Blood".into(),
                description: "Defeat your first enemy.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::DefeatEnemies(1),
                points: 5,
            },
            AchievementDefinition {
                id: "warrior".into(),
                name: "Warrior".into(),
                description: "Defeat 50 enemies.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::DefeatEnemies(50),
                points: 15,
            },
            AchievementDefinition {
                id: "slayer".into(),
                name: "Slayer".into(),
                description: "Defeat 200 enemies.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::DefeatEnemies(200),
                points: 25,
            },
            AchievementDefinition {
                id: "legend_of_battle".into(),
                name: "Legend of Battle".into(),
                description: "Defeat 500 enemies.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::DefeatEnemies(500),
                points: 50,
            },
            AchievementDefinition {
                id: "sword_master".into(),
                name: "Sword Master".into(),
                description: "Use the sword 100 times.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::UseAbility { ability: "sword".into(), count: 100 },
                points: 20,
            },
            AchievementDefinition {
                id: "marksman".into(),
                name: "Marksman".into(),
                description: "Use the pistol 100 times.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::UseAbility { ability: "pistol".into(), count: 100 },
                points: 20,
            },
            AchievementDefinition {
                id: "dynamite_expert".into(),
                name: "Dynamite Expert".into(),
                description: "Use dynamite 50 times.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::UseAbility { ability: "dynamite".into(), count: 50 },
                points: 20,
            },
            AchievementDefinition {
                id: "magic_wielder".into(),
                name: "Magic Wielder".into(),
                description: "Use the Magic Claw 50 times.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::UseAbility { ability: "magic_claw".into(), count: 50 },
                points: 20,
            },
            // --- Boss achievements ---
            AchievementDefinition {
                id: "boss_le_rauxe".into(),
                name: "Boss Slayer: Le Rauxe".into(),
                description: "Defeat Le Rauxe.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::DefeatBoss("le_rauxe".into()),
                points: 25,
            },
            AchievementDefinition {
                id: "boss_katherine".into(),
                name: "Boss Slayer: Katherine".into(),
                description: "Defeat Katherine.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::DefeatBoss("katherine".into()),
                points: 25,
            },
            AchievementDefinition {
                id: "boss_wolvington".into(),
                name: "Boss Slayer: Wolvington".into(),
                description: "Defeat Wolvington.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::DefeatBoss("wolvington".into()),
                points: 25,
            },
            AchievementDefinition {
                id: "boss_omar".into(),
                name: "Boss Slayer: Omar".into(),
                description: "Defeat Omar.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Combat,
                criteria: AchievementCriteria::DefeatBoss("omar".into()),
                points: 25,
            },
            // --- Progression: levels 1-14 ---
            AchievementDefinition {
                id: "level_1".into(),
                name: "Setting Sail".into(),
                description: "Complete Level 1.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(1),
                points: 10,
            },
            AchievementDefinition {
                id: "level_2".into(),
                name: "La Roca".into(),
                description: "Complete Level 2.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(2),
                points: 10,
            },
            AchievementDefinition {
                id: "level_3".into(),
                name: "Underground Passage".into(),
                description: "Complete Level 3.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(3),
                points: 10,
            },
            AchievementDefinition {
                id: "level_4".into(),
                name: "Dark Forest".into(),
                description: "Complete Level 4.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(4),
                points: 10,
            },
            AchievementDefinition {
                id: "level_5".into(),
                name: "Town Center".into(),
                description: "Complete Level 5.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(5),
                points: 10,
            },
            AchievementDefinition {
                id: "level_6".into(),
                name: "Puerto Lobos".into(),
                description: "Complete Level 6.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(6),
                points: 10,
            },
            AchievementDefinition {
                id: "level_7".into(),
                name: "Docks".into(),
                description: "Complete Level 7.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(7),
                points: 10,
            },
            AchievementDefinition {
                id: "level_8".into(),
                name: "Shipyard".into(),
                description: "Complete Level 8.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(8),
                points: 10,
            },
            AchievementDefinition {
                id: "level_9".into(),
                name: "Pirate's Cove".into(),
                description: "Complete Level 9.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(9),
                points: 10,
            },
            AchievementDefinition {
                id: "level_10".into(),
                name: "Crow's Nest".into(),
                description: "Complete Level 10.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(10),
                points: 10,
            },
            AchievementDefinition {
                id: "level_11".into(),
                name: "Cliffs".into(),
                description: "Complete Level 11.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(11),
                points: 10,
            },
            AchievementDefinition {
                id: "level_12".into(),
                name: "Castle Ruins".into(),
                description: "Complete Level 12.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(12),
                points: 10,
            },
            AchievementDefinition {
                id: "level_13".into(),
                name: "Dungeon".into(),
                description: "Complete Level 13.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(13),
                points: 10,
            },
            AchievementDefinition {
                id: "level_14".into(),
                name: "Throne Room".into(),
                description: "Complete Level 14 and finish the game!".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Progression,
                criteria: AchievementCriteria::CompleteLevels(14),
                points: 50,
            },
            // --- Collection ---
            AchievementDefinition {
                id: "treasure_hunter".into(),
                name: "Treasure Hunter".into(),
                description: "Collect 100 treasures.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Collection,
                criteria: AchievementCriteria::CollectTreasures(100),
                points: 15,
            },
            AchievementDefinition {
                id: "treasure_hoarder".into(),
                name: "Treasure Hoarder".into(),
                description: "Collect 500 treasures.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Collection,
                criteria: AchievementCriteria::CollectTreasures(500),
                points: 30,
            },
            AchievementDefinition {
                id: "completionist_level_1".into(),
                name: "Completionist: Level 1".into(),
                description: "Collect every treasure in Level 1.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Collection,
                criteria: AchievementCriteria::CollectAllInLevel(1),
                points: 20,
            },
            // --- Speedrun ---
            AchievementDefinition {
                id: "speed_demon_1".into(),
                name: "Speed Demon: Level 1".into(),
                description: "Complete Level 1 in under 120 seconds.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Speedrun,
                criteria: AchievementCriteria::CompleteUnderTime { level: 1, seconds: 120 },
                points: 30,
            },
            AchievementDefinition {
                id: "speed_demon_5".into(),
                name: "Speed Demon: Level 5".into(),
                description: "Complete Level 5 in under 180 seconds.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Speedrun,
                criteria: AchievementCriteria::CompleteUnderTime { level: 5, seconds: 180 },
                points: 30,
            },
            // --- Challenge ---
            AchievementDefinition {
                id: "untouchable".into(),
                name: "Untouchable".into(),
                description: "Complete the game without dying.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Challenge,
                criteria: AchievementCriteria::CompleteWithoutDying,
                points: 100,
            },
            AchievementDefinition {
                id: "high_score".into(),
                name: "High Score".into(),
                description: "Reach a score of 1,000,000.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Challenge,
                criteria: AchievementCriteria::ReachScore(1_000_000),
                points: 50,
            },
            // --- Exploration / Secrets ---
            AchievementDefinition {
                id: "master_explorer".into(),
                name: "Master Explorer".into(),
                description: "Find 10 secret areas.".into(),
                icon: None,
                hidden: false,
                category: AchievementCategory::Exploration,
                criteria: AchievementCriteria::FindSecret("any_10".into()),
                points: 25,
            },
            AchievementDefinition {
                id: "secret_grotto".into(),
                name: "Secret Grotto".into(),
                description: "Find the hidden grotto in Level 3.".into(),
                icon: None,
                hidden: true,
                category: AchievementCategory::Secret,
                criteria: AchievementCriteria::FindSecret("grotto_level_3".into()),
                points: 15,
            },
            AchievementDefinition {
                id: "hidden_passage".into(),
                name: "Hidden Passage".into(),
                description: "Discover the hidden passage in Level 7.".into(),
                icon: None,
                hidden: true,
                category: AchievementCategory::Secret,
                criteria: AchievementCriteria::FindSecret("passage_level_7".into()),
                points: 15,
            },
        ]
    }

    /// Update progress for all achievements whose criteria match the given kind.
    pub fn check_progress(&mut self, criteria: &AchievementCriteria, current_value: u32) {
        let matching_ids: Vec<String> = self
            .definitions
            .iter()
            .filter(|def| Self::criteria_matches(&def.criteria, criteria))
            .map(|def| def.id.clone())
            .collect();

        for id in matching_ids {
            if let Some(prog) = self.progress.get_mut(&id) {
                if prog.unlocked {
                    continue;
                }
                prog.current_value = current_value;
                prog.progress = if prog.target_value > 0 {
                    (current_value as f32 / prog.target_value as f32).min(1.0)
                } else {
                    1.0
                };
                if prog.current_value >= prog.target_value {
                    Self::unlock_inner(&id, prog);
                    self.notification_queue.push(id.clone());
                    if let Some(def) = self.definitions.iter().find(|d| d.id == id) {
                        self.total_points += def.points;
                    }
                }
            }
        }
    }

    /// Force-unlock an achievement by ID.
    pub fn unlock(&mut self, id: &str) {
        if let Some(prog) = self.progress.get_mut(id) {
            if prog.unlocked {
                return;
            }
            Self::unlock_inner(id, prog);
            self.notification_queue.push(id.to_string());
            if let Some(def) = self.definitions.iter().find(|d| d.id == id) {
                self.total_points += def.points;
            }
        }
    }

    /// Return whether the given achievement has been unlocked.
    pub fn is_unlocked(&self, id: &str) -> bool {
        self.progress
            .get(id)
            .map(|p| p.unlocked)
            .unwrap_or(false)
    }

    /// Return the progress for a given achievement.
    pub fn get_progress(&self, id: &str) -> Option<&AchievementProgress> {
        self.progress.get(id)
    }

    /// Drain the notification queue and return the definitions that should be shown.
    pub fn pending_notifications(&mut self) -> Vec<AchievementDefinition> {
        let ids: Vec<String> = self.notification_queue.drain(..).collect();
        ids.iter()
            .filter_map(|id| {
                if let Some(prog) = self.progress.get_mut(id) {
                    prog.notified = true;
                }
                self.definitions.iter().find(|d| d.id == *id).cloned()
            })
            .collect()
    }

    /// Persist achievement progress to a JSON file.
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.progress)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    /// Load achievement progress from a JSON file.
    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        let loaded: HashMap<String, AchievementProgress> = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        // Merge loaded progress with existing definitions
        for (id, prog) in loaded {
            if self.progress.contains_key(&id) {
                self.progress.insert(id, prog);
            }
        }
        // Recalculate total points
        self.total_points = 0;
        for def in &self.definitions {
            if self
                .progress
                .get(&def.id)
                .map(|p| p.unlocked)
                .unwrap_or(false)
            {
                self.total_points += def.points;
            }
        }
        Ok(())
    }

    /// Overall completion percentage across all achievements.
    pub fn completion_percentage(&self) -> f32 {
        if self.definitions.is_empty() {
            return 0.0;
        }
        let unlocked = self.get_unlocked_count() as f32;
        let total = self.get_total_count() as f32;
        (unlocked / total) * 100.0
    }

    /// Number of unlocked achievements.
    pub fn get_unlocked_count(&self) -> usize {
        self.progress.values().filter(|p| p.unlocked).count()
    }

    /// Total number of defined achievements.
    pub fn get_total_count(&self) -> usize {
        self.definitions.len()
    }

    // ---- private helpers ----

    fn unlock_inner(_id: &str, prog: &mut AchievementProgress) {
        prog.unlocked = true;
        prog.progress = 1.0;
        prog.current_value = prog.target_value;
        // Simple timestamp: we avoid pulling in chrono by storing a placeholder.
        prog.unlock_time = Some("unlocked".to_string());
    }

    /// Derive the numeric target value from an achievement criteria.
    fn target_for_criteria(criteria: &AchievementCriteria) -> u32 {
        match criteria {
            AchievementCriteria::CompleteLevels(n) => *n,
            AchievementCriteria::DefeatEnemies(n) => *n,
            AchievementCriteria::DefeatBoss(_) => 1,
            AchievementCriteria::CollectTreasures(n) => *n,
            AchievementCriteria::CollectAllInLevel(_) => 1,
            AchievementCriteria::CompleteWithoutDying => 1,
            AchievementCriteria::CompleteUnderTime { .. } => 1,
            AchievementCriteria::FindSecret(_) => 1,
            AchievementCriteria::ReachScore(_) => 1,
            AchievementCriteria::UseAbility { count, .. } => *count,
            AchievementCriteria::Custom(_) => 1,
        }
    }

    /// Check whether two criteria are of the same *kind* for progress matching.
    fn criteria_matches(definition: &AchievementCriteria, event: &AchievementCriteria) -> bool {
        use AchievementCriteria::*;
        match (definition, event) {
            (CompleteLevels(_), CompleteLevels(_)) => true,
            (DefeatEnemies(_), DefeatEnemies(_)) => true,
            (DefeatBoss(a), DefeatBoss(b)) => a == b,
            (CollectTreasures(_), CollectTreasures(_)) => true,
            (CollectAllInLevel(a), CollectAllInLevel(b)) => a == b,
            (CompleteWithoutDying, CompleteWithoutDying) => true,
            (
                CompleteUnderTime {
                    level: la,
                    seconds: _,
                },
                CompleteUnderTime {
                    level: lb,
                    seconds: _,
                },
            ) => la == lb,
            (FindSecret(a), FindSecret(b)) => a == b,
            (ReachScore(_), ReachScore(_)) => true,
            (
                UseAbility {
                    ability: a,
                    count: _,
                },
                UseAbility {
                    ability: b,
                    count: _,
                },
            ) => a == b,
            (Custom(a), Custom(b)) => a == b,
            _ => false,
        }
    }
}

impl System for AchievementSystem {
    fn name(&self) -> &str {
        "AchievementSystem"
    }

    fn update(&mut self, _world: &mut World, dt: f32) {
        // Tick the notification display timer
        if self.notification_timer > 0.0 {
            self.notification_timer -= dt;
        }
    }
}
