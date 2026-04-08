//! State synchronisation — client-side prediction, server reconciliation, and
//! entity interpolation.

use std::collections::{HashMap, VecDeque};

use crate::protocol::{EntitySnapshot, PlayerInputData};

// ---------------------------------------------------------------------------
// Client-side prediction
// ---------------------------------------------------------------------------

/// A single predicted input that has not yet been acknowledged by the server.
#[derive(Debug, Clone)]
pub struct PendingInput {
    pub tick: u64,
    pub input: PlayerInputData,
    /// The predicted position after applying this input.
    pub predicted_x: f32,
    pub predicted_y: f32,
}

/// Client-side prediction buffer — stores unacknowledged inputs so that they
/// can be replayed after receiving an authoritative server snapshot.
pub struct ClientPrediction {
    /// Inputs that have been sent but not yet confirmed.
    pub pending: VecDeque<PendingInput>,
    /// Last server-confirmed tick.
    pub last_confirmed_tick: u64,
    /// Player's last confirmed position.
    pub confirmed_x: f32,
    pub confirmed_y: f32,
}

impl ClientPrediction {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            last_confirmed_tick: 0,
            confirmed_x: 0.0,
            confirmed_y: 0.0,
        }
    }

    /// Record a new input that was just sent to the server.
    pub fn push_input(&mut self, tick: u64, input: PlayerInputData, pred_x: f32, pred_y: f32) {
        self.pending.push_back(PendingInput {
            tick,
            input,
            predicted_x: pred_x,
            predicted_y: pred_y,
        });
    }

    /// Reconcile with an authoritative snapshot. Discards all inputs up to and
    /// including `server_tick`, then replays remaining inputs on top of the
    /// confirmed position.
    pub fn reconcile(
        &mut self,
        server_tick: u64,
        server_x: f32,
        server_y: f32,
        apply_input: impl Fn(f32, f32, &PlayerInputData) -> (f32, f32),
    ) -> (f32, f32) {
        self.last_confirmed_tick = server_tick;
        self.confirmed_x = server_x;
        self.confirmed_y = server_y;

        // Drop acknowledged inputs.
        while self.pending.front().is_some_and(|p| p.tick <= server_tick) {
            self.pending.pop_front();
        }

        // Replay remaining predicted inputs.
        let mut x = server_x;
        let mut y = server_y;
        for pi in &mut self.pending {
            let (nx, ny) = apply_input(x, y, &pi.input);
            pi.predicted_x = nx;
            pi.predicted_y = ny;
            x = nx;
            y = ny;
        }

        (x, y)
    }
}

impl Default for ClientPrediction {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Entity interpolation
// ---------------------------------------------------------------------------

/// Buffered snapshot for a single remote entity, used for smooth interpolation.
#[derive(Debug, Clone)]
pub struct InterpolationEntry {
    pub tick: u64,
    pub x: f32,
    pub y: f32,
    pub animation: String,
    pub flip_x: bool,
}

/// Interpolation buffer for remote entities — renders them slightly behind real
/// time to smooth out network jitter.
pub struct EntityInterpolator {
    /// Per-entity ring buffer of recent snapshots.
    pub buffers: HashMap<u64, VecDeque<InterpolationEntry>>,
    /// How many ticks behind real-time the interpolation target sits.
    pub delay_ticks: u64,
    /// Maximum buffered snapshots per entity.
    pub max_buffer_size: usize,
}

impl EntityInterpolator {
    pub fn new(delay_ticks: u64) -> Self {
        Self {
            buffers: HashMap::new(),
            delay_ticks,
            max_buffer_size: 30,
        }
    }

    /// Feed a new authoritative snapshot into the buffer.
    pub fn push_snapshot(&mut self, snapshot: &EntitySnapshot) {
        let buf = self.buffers.entry(snapshot.entity_id).or_default();
        buf.push_back(InterpolationEntry {
            tick: 0, // caller should set the tick
            x: snapshot.x,
            y: snapshot.y,
            animation: snapshot.animation.clone(),
            flip_x: snapshot.flip_x,
        });
        if buf.len() > self.max_buffer_size {
            buf.pop_front();
        }
    }

    /// Push a snapshot with an explicit tick.
    pub fn push_snapshot_with_tick(&mut self, tick: u64, snapshot: &EntitySnapshot) {
        let buf = self.buffers.entry(snapshot.entity_id).or_default();
        buf.push_back(InterpolationEntry {
            tick,
            x: snapshot.x,
            y: snapshot.y,
            animation: snapshot.animation.clone(),
            flip_x: snapshot.flip_x,
        });
        if buf.len() > self.max_buffer_size {
            buf.pop_front();
        }
    }

    /// Interpolate the position of `entity_id` at the given fractional tick.
    /// Returns `None` if there are not enough samples.
    pub fn interpolate(&self, entity_id: u64, render_tick: f64) -> Option<(f32, f32)> {
        let buf = self.buffers.get(&entity_id)?;
        if buf.len() < 2 {
            return buf.back().map(|e| (e.x, e.y));
        }

        // Find the two entries that straddle `render_tick`.
        let mut prev = &buf[0];
        for entry in buf.iter().skip(1) {
            if entry.tick as f64 >= render_tick {
                let range = (entry.tick as f64 - prev.tick as f64).max(1.0);
                let t = ((render_tick - prev.tick as f64) / range).clamp(0.0, 1.0) as f32;
                let x = prev.x + (entry.x - prev.x) * t;
                let y = prev.y + (entry.y - prev.y) * t;
                return Some((x, y));
            }
            prev = entry;
        }

        // Past all entries — extrapolate from the last two.
        buf.back().map(|e| (e.x, e.y))
    }

    /// Remove interpolation data for a destroyed entity.
    pub fn remove_entity(&mut self, entity_id: u64) {
        self.buffers.remove(&entity_id);
    }
}

impl Default for EntityInterpolator {
    fn default() -> Self {
        Self::new(3)
    }
}
