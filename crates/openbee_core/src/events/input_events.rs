//! User input events (CLI, keyboard, mouse — generic).

use serde::{Deserialize, Serialize};

use crate::events::event_bus::Event;

/// User entered a command in the interactive REPL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntered {
    pub command: String,
}
impl Event for CommandEntered {}

/// User confirmed a security-sensitive action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfirmed {
    pub action: String,
}
impl Event for UserConfirmed {}

/// User cancelled an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCancelled {
    pub action: String,
}
impl Event for UserCancelled {}
