//! Task executor — picks tasks from the queue and runs them.

use crate::skill::Skill;
use crate::task::{Task, TaskResult, TaskStatus};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, info};

/// The executor routes tasks to skills and manages execution.
pub struct TaskExecutor {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Register a skill with the executor.
    pub fn register_skill(&mut self, skill: Box<dyn Skill>) {
        info!(
            "Registered skill: {} — {}",
            skill.name(),
            skill.description()
        );
        self.skills.insert(skill.name().to_string(), skill);
    }

    /// Find the best skill to handle an instruction.
    pub fn route(&self, instruction: &str) -> Option<(&str, f32)> {
        let mut best: Option<(&str, f32)> = None;

        for skill in self.skills.values() {
            let score = skill.can_handle(instruction);
            if score > 0.0 && (best.is_none() || score > best.unwrap().1) {
                best = Some((skill.name(), score));
            }
        }

        best
    }

    /// Execute a task by routing it to the appropriate skill.
    pub fn execute(&self, task: &mut Task) -> TaskResult {
        let start = Instant::now();
        task.started_instant = Some(start);
        task.status = TaskStatus::Running {
            progress: 0.0,
            message: "Starting...".to_string(),
        };

        // Find the skill
        let skill_name = if let Some(ref name) = task.skill_name {
            name.clone()
        } else if let Some((name, _score)) = self.route(&task.instruction) {
            name.to_string()
        } else {
            let result = TaskResult::failure(format!(
                "No skill found for instruction: '{}'. Available skills: {}",
                task.instruction,
                self.list_skills().join(", ")
            ));
            task.status = TaskStatus::Failed {
                error: result.error.clone().unwrap_or_default(),
            };
            task.result = Some(result.clone());
            return result;
        };

        let skill = match self.skills.get(&skill_name) {
            Some(s) => s,
            None => {
                let result = TaskResult::failure(format!("Skill '{}' not found", skill_name));
                task.status = TaskStatus::Failed {
                    error: result.error.clone().unwrap_or_default(),
                };
                task.result = Some(result.clone());
                return result;
            }
        };

        info!("Executing task #{} with skill '{}'", task.id, skill_name);

        // Execute
        let mut result = skill.execute(task);
        result.duration_ms = start.elapsed().as_millis() as u64;

        // Update task
        if result.success {
            task.status = TaskStatus::Completed;
            info!(
                "Task #{} completed in {}ms: {}",
                task.id, result.duration_ms, result.summary
            );
        } else {
            task.status = TaskStatus::Failed {
                error: result.error.clone().unwrap_or_default(),
            };
            error!(
                "Task #{} failed after {}ms: {}",
                task.id,
                result.duration_ms,
                result.error.as_deref().unwrap_or("unknown error")
            );
        }

        task.result = Some(result.clone());
        task.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );

        result
    }

    /// List all registered skill names.
    pub fn list_skills(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Get a specific skill by name.
    pub fn get_skill(&self, name: &str) -> Option<&dyn Skill> {
        self.skills.get(name).map(|s| s.as_ref())
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}
