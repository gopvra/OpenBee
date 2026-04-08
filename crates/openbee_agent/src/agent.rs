//! The main BeeAgent — accepts instructions, routes to skills, manages history.

use crate::executor::TaskExecutor;
use crate::skill::Skill;
use crate::task::{Task, TaskId, TaskPriority, TaskResult, TaskStatus};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::info;

/// Configuration for the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum number of tasks to keep in history.
    pub max_history: usize,
    /// Whether to auto-execute tasks or require manual trigger.
    pub auto_execute: bool,
    /// Agent name for display.
    pub name: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            auto_execute: true,
            name: "Bee".to_string(),
        }
    }
}

/// The main AI Agent that accepts tasks and executes them.
pub struct BeeAgent {
    executor: TaskExecutor,
    config: AgentConfig,
    /// Task queue (pending tasks).
    queue: VecDeque<Task>,
    /// Completed task history.
    history: VecDeque<Task>,
    /// Next task ID.
    next_id: TaskId,
}

impl BeeAgent {
    /// Create a new agent with default configuration.
    pub fn new() -> Self {
        Self::with_config(AgentConfig::default())
    }

    /// Create a new agent with custom configuration.
    pub fn with_config(config: AgentConfig) -> Self {
        info!("BeeAgent '{}' initialized", config.name);
        Self {
            executor: TaskExecutor::new(),
            config,
            queue: VecDeque::new(),
            history: VecDeque::new(),
            next_id: 1,
        }
    }

    /// Register a skill the agent can use.
    pub fn register_skill(&mut self, skill: Box<dyn Skill>) {
        self.executor.register_skill(skill);
    }

    /// Submit a task from a natural language instruction.
    /// Returns the task ID immediately.
    pub fn submit(&mut self, instruction: impl Into<String>) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;

        let instruction = instruction.into();
        info!("Task #{} submitted: '{}'", id, instruction);

        let mut task = Task::new(id, &instruction);

        // Try to route to a skill
        if let Some((skill_name, score)) = self.executor.route(&instruction) {
            task.skill_name = Some(skill_name.to_string());
            info!(
                "Task #{} routed to skill '{}' (confidence: {:.0}%)",
                id,
                skill_name,
                score * 100.0
            );
        }

        if self.config.auto_execute {
            // Execute immediately
            self.executor.execute(&mut task);
            self.push_history(task.clone());
        } else {
            self.queue.push_back(task);
        }

        id
    }

    /// Submit a task with specific priority.
    pub fn submit_priority(
        &mut self,
        instruction: impl Into<String>,
        priority: TaskPriority,
    ) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;

        let instruction = instruction.into();
        let mut task = Task::new(id, &instruction).with_priority(priority);

        if let Some((skill_name, _)) = self.executor.route(&instruction) {
            task.skill_name = Some(skill_name.to_string());
        }

        if self.config.auto_execute {
            self.executor.execute(&mut task);
            self.push_history(task.clone());
        } else {
            // Insert by priority (higher priority first)
            let pos = self
                .queue
                .iter()
                .position(|t| t.priority < priority)
                .unwrap_or(self.queue.len());
            self.queue.insert(pos, task);
        }

        id
    }

    /// Execute the next queued task (manual mode).
    pub fn execute_next(&mut self) -> Option<TaskResult> {
        let mut task = self.queue.pop_front()?;
        let result = self.executor.execute(&mut task);
        self.push_history(task);
        Some(result)
    }

    /// Execute all queued tasks.
    pub fn execute_all(&mut self) -> Vec<TaskResult> {
        let mut results = Vec::new();
        while let Some(result) = self.execute_next() {
            results.push(result);
        }
        results
    }

    /// Get the result of a completed task by ID.
    pub fn get_result(&self, id: TaskId) -> Option<&TaskResult> {
        self.history
            .iter()
            .find(|t| t.id == id)
            .and_then(|t| t.result.as_ref())
    }

    /// Get a task by ID (from queue or history).
    pub fn get_task(&self, id: TaskId) -> Option<&Task> {
        self.queue
            .iter()
            .chain(self.history.iter())
            .find(|t| t.id == id)
    }

    /// Get the current task queue.
    pub fn queue(&self) -> &VecDeque<Task> {
        &self.queue
    }

    /// Get completed task history.
    pub fn history(&self) -> &VecDeque<Task> {
        &self.history
    }

    /// Cancel a queued task.
    pub fn cancel(&mut self, id: TaskId) -> bool {
        if let Some(pos) = self.queue.iter().position(|t| t.id == id) {
            let mut task = self.queue.remove(pos).unwrap();
            task.status = TaskStatus::Cancelled;
            info!("Task #{} cancelled", id);
            self.push_history(task);
            true
        } else {
            false
        }
    }

    /// List all available skills and their commands.
    pub fn list_skills(&self) -> Vec<(&str, &str)> {
        self.executor
            .list_skills()
            .iter()
            .filter_map(|name| {
                self.executor
                    .get_skill(name)
                    .map(|s| (*name, s.description()))
            })
            .collect()
    }

    /// Show help — what the agent can do.
    pub fn help(&self) -> String {
        let mut help = String::new();
        help.push_str(&format!(
            "🐝 {} Agent — Available Skills:\n\n",
            self.config.name
        ));

        for name in self.executor.list_skills() {
            if let Some(skill) = self.executor.get_skill(name) {
                help.push_str(&format!("  [{}] {}\n", name, skill.description()));
                for cmd in skill.commands() {
                    help.push_str(&format!("    • {} — {}\n", cmd.name, cmd.description));
                    help.push_str(&format!("      Example: {}\n", cmd.example));
                }
                help.push('\n');
            }
        }

        help.push_str(&format!(
            "Queued: {} | Completed: {} | Skills: {}\n",
            self.queue.len(),
            self.history.len(),
            self.executor.list_skills().len()
        ));

        help
    }

    /// Get stats about the agent's activity.
    pub fn stats(&self) -> AgentStats {
        let total = self.history.len();
        let succeeded = self
            .history
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let failed = self
            .history
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Failed { .. }))
            .count();
        let avg_duration = if total > 0 {
            self.history
                .iter()
                .filter_map(|t| t.result.as_ref().map(|r| r.duration_ms))
                .sum::<u64>()
                / total as u64
        } else {
            0
        };

        AgentStats {
            total_tasks: total,
            succeeded,
            failed,
            queued: self.queue.len(),
            avg_duration_ms: avg_duration,
            skills_registered: self.executor.list_skills().len(),
        }
    }

    fn push_history(&mut self, task: Task) {
        self.history.push_back(task);
        while self.history.len() > self.config.max_history {
            self.history.pop_front();
        }
    }
}

impl Default for BeeAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics about agent activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub total_tasks: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub queued: usize,
    pub avg_duration_ms: u64,
    pub skills_registered: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::SkillCommand;

    struct EchoSkill;

    impl Skill for EchoSkill {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Echoes back the instruction"
        }
        fn keywords(&self) -> &[&str] {
            &["echo", "say", "repeat"]
        }
        fn execute(&self, task: &Task) -> TaskResult {
            TaskResult::success(format!("Echo: {}", task.instruction))
        }
        fn commands(&self) -> Vec<SkillCommand> {
            vec![SkillCommand {
                name: "echo".to_string(),
                description: "Repeat what you said".to_string(),
                example: "echo hello world".to_string(),
            }]
        }
    }

    #[test]
    fn test_agent_submit_and_execute() {
        let mut agent = BeeAgent::new();
        agent.register_skill(Box::new(EchoSkill));

        let id = agent.submit("echo hello");
        let result = agent.get_result(id).unwrap();
        assert!(result.success);
        assert!(result.summary.contains("Echo: echo hello"));
    }

    #[test]
    fn test_agent_no_matching_skill() {
        let mut agent = BeeAgent::new();
        agent.register_skill(Box::new(EchoSkill));

        let id = agent.submit("do something completely unrelated xyz");
        let result = agent.get_result(id).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_agent_manual_mode() {
        let mut agent = BeeAgent::with_config(AgentConfig {
            auto_execute: false,
            ..Default::default()
        });
        agent.register_skill(Box::new(EchoSkill));

        let id = agent.submit("echo test");
        // Should be queued, not executed
        assert!(agent.get_result(id).is_none());
        assert_eq!(agent.queue().len(), 1);

        // Execute manually
        let result = agent.execute_next().unwrap();
        assert!(result.success);
        assert_eq!(agent.queue().len(), 0);
    }

    #[test]
    fn test_agent_cancel() {
        let mut agent = BeeAgent::with_config(AgentConfig {
            auto_execute: false,
            ..Default::default()
        });
        agent.register_skill(Box::new(EchoSkill));

        let id = agent.submit("echo cancel me");
        assert!(agent.cancel(id));
        assert_eq!(agent.queue().len(), 0);

        let task = agent.get_task(id).unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[test]
    fn test_agent_stats() {
        let mut agent = BeeAgent::new();
        agent.register_skill(Box::new(EchoSkill));

        agent.submit("echo one");
        agent.submit("echo two");
        agent.submit("unknown xyz");

        let stats = agent.stats();
        assert_eq!(stats.total_tasks, 3);
        assert_eq!(stats.succeeded, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.skills_registered, 1);
    }
}
