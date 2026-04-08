//! Deferred process system for time-based game logic (powerups, timers, etc.).

pub mod manager;
pub mod powerup;
#[allow(clippy::module_inception)]
pub mod process;

pub use manager::ProcessManager;
pub use powerup::PowerupProcess;
pub use process::{Process, ProcessId, ProcessState};
