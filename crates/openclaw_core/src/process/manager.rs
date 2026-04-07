//! Process manager for running and chaining deferred processes.

use std::collections::HashMap;

use super::process::{Process, ProcessId, ProcessState};

/// Entry in the process manager.
struct ProcessEntry {
    process: Box<dyn Process>,
    state: ProcessState,
    /// Child process to start when this one succeeds.
    child: Option<Box<dyn Process>>,
}

/// Manages a set of deferred processes, updating them each frame and supporting
/// process chains (a child process starts when its parent succeeds).
pub struct ProcessManager {
    processes: HashMap<ProcessId, ProcessEntry>,
    next_id: u64,
}

impl ProcessManager {
    /// Create a new empty process manager.
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a process to the manager. Calls `on_init()` immediately. Returns the process ID.
    pub fn add_process(&mut self, mut process: Box<dyn Process>) -> ProcessId {
        let id = ProcessId(self.next_id);
        self.next_id += 1;
        process.on_init();
        tracing::debug!("ProcessManager: added process '{}' (id={:?})", process.name(), id);
        self.processes.insert(
            id,
            ProcessEntry {
                process,
                state: ProcessState::Running,
                child: None,
            },
        );
        id
    }

    /// Attach a child process that will start when the parent succeeds.
    pub fn attach_child(&mut self, parent_id: ProcessId, child: Box<dyn Process>) {
        if let Some(entry) = self.processes.get_mut(&parent_id) {
            entry.child = Some(child);
        } else {
            tracing::warn!(
                "ProcessManager: cannot attach child to non-existent process {:?}",
                parent_id
            );
        }
    }

    /// Abort a process by its ID.
    pub fn abort(&mut self, id: ProcessId) {
        if let Some(entry) = self.processes.get_mut(&id) {
            if entry.state == ProcessState::Running {
                entry.state = ProcessState::Aborted;
                entry.process.on_abort();
                tracing::debug!("ProcessManager: aborted process {:?}", id);
            }
        }
    }

    /// Update all running processes. Removes finished processes and starts children of succeeded ones.
    pub fn update_all(&mut self, dt: f32) {
        let mut children_to_add = Vec::new();
        let mut to_remove = Vec::new();

        for (&id, entry) in self.processes.iter_mut() {
            if entry.state != ProcessState::Running {
                continue;
            }

            let new_state = entry.process.update(dt);
            entry.state = new_state;

            match new_state {
                ProcessState::Succeeded => {
                    entry.process.on_success();
                    if let Some(child) = entry.child.take() {
                        children_to_add.push(child);
                    }
                    to_remove.push(id);
                }
                ProcessState::Failed | ProcessState::Aborted => {
                    if new_state == ProcessState::Aborted {
                        entry.process.on_abort();
                    }
                    to_remove.push(id);
                }
                ProcessState::Running => {}
            }
        }

        // Remove finished processes.
        for id in to_remove {
            self.processes.remove(&id);
        }

        // Start child processes.
        for child in children_to_add {
            self.add_process(child);
        }
    }

    /// Return the number of currently active (running) processes.
    pub fn active_count(&self) -> usize {
        self.processes
            .values()
            .filter(|e| e.state == ProcessState::Running)
            .count()
    }

    /// Return the total number of processes (including finished ones not yet cleaned up).
    pub fn total_count(&self) -> usize {
        self.processes.len()
    }

    /// Check if a specific process is still running.
    pub fn is_running(&self, id: ProcessId) -> bool {
        self.processes
            .get(&id)
            .map_or(false, |e| e.state == ProcessState::Running)
    }

    /// Clear all processes.
    pub fn clear(&mut self) {
        for entry in self.processes.values_mut() {
            if entry.state == ProcessState::Running {
                entry.process.on_abort();
            }
        }
        self.processes.clear();
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
