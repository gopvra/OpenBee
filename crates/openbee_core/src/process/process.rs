//! Process trait and state for deferred game logic.

/// Unique identifier for a process within the process manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(pub u64);

/// Current state of a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// The process is still running and should continue to be updated.
    Running,
    /// The process completed successfully.
    Succeeded,
    /// The process failed.
    Failed,
    /// The process was externally aborted.
    Aborted,
}

/// Trait for deferred processes that run over multiple frames.
pub trait Process: Send + Sync {
    /// Called once when the process is first added to the manager.
    fn on_init(&mut self) {}

    /// Called each frame with the delta time in seconds. Return the new process state.
    fn update(&mut self, dt: f32) -> ProcessState;

    /// Called when the process completes successfully.
    fn on_success(&mut self) {}

    /// Called when the process is aborted.
    fn on_abort(&mut self) {}

    /// Human-readable name for debugging.
    fn name(&self) -> &str {
        "Process"
    }
}
