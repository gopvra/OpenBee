//! Simple profiling utilities for measuring frame and system times.

use std::sync::Mutex;
use std::time::Instant;

/// Statistics collected by the profiler.
#[derive(Debug, Clone, Copy)]
pub struct ProfilerStats {
    /// Total frame time in seconds.
    pub frame_time: f64,
    /// Frames per second.
    pub fps: f64,
    /// Time spent in the update phase (seconds).
    pub update_time: f64,
    /// Time spent in the render phase (seconds).
    pub render_time: f64,
}

impl Default for ProfilerStats {
    fn default() -> Self {
        Self {
            frame_time: 0.0,
            fps: 0.0,
            update_time: 0.0,
            render_time: 0.0,
        }
    }
}

/// Global profiler that collects timing statistics.
pub struct Profiler {
    stats: ProfilerStats,
    frame_start: Instant,
    update_start: Option<Instant>,
    render_start: Option<Instant>,
    /// Exponential moving average smoothing factor.
    smoothing: f64,
}

/// Global profiler instance behind a mutex for thread safety.
static GLOBAL_PROFILER: Mutex<Option<Profiler>> = Mutex::new(None);

impl Profiler {
    /// Create a new profiler.
    pub fn new() -> Self {
        Self {
            stats: ProfilerStats::default(),
            frame_start: Instant::now(),
            update_start: None,
            render_start: None,
            smoothing: 0.9,
        }
    }

    /// Initialize the global profiler singleton.
    pub fn init_global() {
        let mut lock = GLOBAL_PROFILER.lock().unwrap();
        *lock = Some(Profiler::new());
    }

    /// Access the global profiler, running a closure with a mutable reference.
    pub fn with_global<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&mut Profiler) -> R,
    {
        let mut lock = GLOBAL_PROFILER.lock().ok()?;
        lock.as_mut().map(f)
    }

    /// Mark the start of a new frame.
    pub fn begin_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    /// Mark the end of the frame and compute statistics.
    pub fn end_frame(&mut self) {
        let frame_time = self.frame_start.elapsed().as_secs_f64();
        let s = self.smoothing;
        self.stats.frame_time = self.stats.frame_time * s + frame_time * (1.0 - s);
        self.stats.fps = if self.stats.frame_time > 0.0 {
            1.0 / self.stats.frame_time
        } else {
            0.0
        };
    }

    /// Mark the start of the update phase.
    pub fn begin_update(&mut self) {
        self.update_start = Some(Instant::now());
    }

    /// Mark the end of the update phase.
    pub fn end_update(&mut self) {
        if let Some(start) = self.update_start.take() {
            let elapsed = start.elapsed().as_secs_f64();
            let s = self.smoothing;
            self.stats.update_time = self.stats.update_time * s + elapsed * (1.0 - s);
        }
    }

    /// Mark the start of the render phase.
    pub fn begin_render(&mut self) {
        self.render_start = Some(Instant::now());
    }

    /// Mark the end of the render phase.
    pub fn end_render(&mut self) {
        if let Some(start) = self.render_start.take() {
            let elapsed = start.elapsed().as_secs_f64();
            let s = self.smoothing;
            self.stats.render_time = self.stats.render_time * s + elapsed * (1.0 - s);
        }
    }

    /// Get the current profiler statistics.
    pub fn stats(&self) -> ProfilerStats {
        self.stats
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII scoped timer that logs its duration when dropped.
pub struct ScopedTimer {
    name: &'static str,
    start: Instant,
}

impl ScopedTimer {
    /// Create a new scoped timer with the given name. Timing starts immediately.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        tracing::trace!("{}: {:.3}ms", self.name, elapsed.as_secs_f64() * 1000.0);
    }
}
