//! Task definition and lifecycle.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Unique task identifier.
pub type TaskId = u64;

/// Priority levels for task scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Current status of a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// Waiting to be picked up by the executor.
    Queued,
    /// Currently being executed.
    Running { progress: f32, message: String },
    /// Completed successfully.
    Completed,
    /// Failed with an error.
    Failed { error: String },
    /// Cancelled by user.
    Cancelled,
}

/// The result of a completed task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Human-readable summary of what was done.
    pub summary: String,
    /// Detailed output data (JSON-serializable).
    pub data: serde_json::Value,
    /// Whether the task succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// How long the task took in milliseconds.
    pub duration_ms: u64,
    /// Suggested follow-up actions.
    pub suggestions: Vec<String>,
}

impl TaskResult {
    pub fn success(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            data: serde_json::Value::Null,
            success: true,
            error: None,
            duration_ms: 0,
            suggestions: Vec::new(),
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn failure(error: impl Into<String>) -> Self {
        let err = error.into();
        Self {
            summary: format!("Task failed: {}", err),
            data: serde_json::Value::Null,
            success: false,
            error: Some(err),
            duration_ms: 0,
            suggestions: Vec::new(),
        }
    }
}

/// A task submitted to the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier.
    pub id: TaskId,
    /// Original user instruction (natural language).
    pub instruction: String,
    /// Which skill should handle this task.
    pub skill_name: Option<String>,
    /// Parsed parameters (extracted from instruction).
    pub params: serde_json::Value,
    /// Current status.
    pub status: TaskStatus,
    /// Result once completed.
    pub result: Option<TaskResult>,
    /// Priority level.
    pub priority: TaskPriority,
    /// Creation timestamp (unix ms).
    pub created_at: u64,
    /// Completion timestamp (unix ms).
    pub completed_at: Option<u64>,
    /// Tags for categorization.
    pub tags: Vec<String>,

    /// Runtime tracking (not serialized).
    #[serde(skip)]
    pub started_instant: Option<Instant>,
}

impl Task {
    /// Create a new task from a user instruction.
    pub fn new(id: TaskId, instruction: impl Into<String>) -> Self {
        Self {
            id,
            instruction: instruction.into(),
            skill_name: None,
            params: serde_json::Value::Null,
            status: TaskStatus::Queued,
            result: None,
            priority: TaskPriority::Normal,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            completed_at: None,
            tags: Vec::new(),
            started_instant: None,
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed | TaskStatus::Failed { .. } | TaskStatus::Cancelled
        )
    }
}
