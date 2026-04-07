//! Boss stager component for managing boss encounter setup.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Which boss encounter to stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BossType {
    /// La Rauxe - Level 2 boss.
    LaRauxe,
    /// Katherine - Level 4 boss.
    Katherine,
    /// Wolvington - Level 6 boss.
    Wolvington,
    /// Gabriel - Level 9 boss.
    Gabriel,
    /// Marrow - Level 11 boss.
    Marrow,
    /// Aquatis - Level 12 boss.
    Aquatis,
    /// Red Tail - Level 13 boss.
    RedTail,
    /// Omar - Level 14 final boss.
    Omar,
}

/// Manages the setup and staging of a boss encounter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossStagerComponent {
    /// Which boss this stager triggers.
    pub boss_type: BossType,
    /// Whether the boss intro cinematic has played.
    pub intro_played: bool,
    /// Music track to play during the boss fight.
    pub music_track: String,
}

impl Default for BossStagerComponent {
    fn default() -> Self {
        Self {
            boss_type: BossType::LaRauxe,
            intro_played: false,
            music_track: String::new(),
        }
    }
}

impl Component for BossStagerComponent {}
