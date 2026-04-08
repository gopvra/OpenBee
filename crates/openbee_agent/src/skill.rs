//! Skill trait — pluggable capabilities the agent can execute.

use crate::task::{Task, TaskResult};

/// A skill is a capability the agent can use to fulfill tasks.
///
/// Each skill declares what kinds of instructions it can handle
/// and provides the execution logic.
pub trait Skill: Send + Sync {
    /// Unique name of this skill (e.g., "wallet", "browser", "system").
    fn name(&self) -> &str;

    /// Human-readable description of what this skill can do.
    fn description(&self) -> &str;

    /// Keywords or patterns this skill responds to.
    /// Used for routing instructions to the right skill.
    fn keywords(&self) -> &[&str];

    /// Check if this skill can handle the given instruction.
    /// Returns a confidence score (0.0 = no match, 1.0 = perfect match).
    fn can_handle(&self, instruction: &str) -> f32 {
        let lower = instruction.to_lowercase();
        let keywords = self.keywords();
        if keywords.is_empty() {
            return 0.0;
        }
        let matches = keywords.iter().filter(|k| lower.contains(*k)).count();
        matches as f32 / keywords.len().max(1) as f32
    }

    /// Execute the task and return the result.
    fn execute(&self, task: &Task) -> TaskResult;

    /// List available sub-commands for this skill.
    fn commands(&self) -> Vec<SkillCommand> {
        Vec::new()
    }
}

/// A command that a skill supports.
#[derive(Debug, Clone)]
pub struct SkillCommand {
    /// Command name (e.g., "balance", "send", "swap").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Usage example.
    pub example: String,
}
