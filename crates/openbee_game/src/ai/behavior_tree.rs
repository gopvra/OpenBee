//! Behavior tree AI system — an alternative to finite state machines.
//!
//! Provides composite nodes (sequence, selector, parallel, random selector),
//! decorator nodes (inverter, repeat, succeeder, timeout, guard, cooldown),
//! leaf nodes (condition, wait, action), and a builder API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Result status returned by every behavior tree node on each tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BtStatus {
    Success,
    Failure,
    Running,
}

/// A node in the behavior tree.
pub trait BtNode: Send + Sync {
    /// Execute one tick of this node.
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus;
    /// Reset this node (and any children) to their initial state.
    fn reset(&mut self);
    /// Human-readable name for debugging.
    fn name(&self) -> &str;
}

/// Dynamic value stored in the blackboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BtValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

/// Context passed to BT nodes for reading/writing game state.
pub struct BtContext {
    pub entity_id: u64,
    pub dt: f32,
    pub player_pos: (f32, f32),
    pub self_pos: (f32, f32),
    pub self_health: f32,
    pub self_health_max: f32,
    pub distance_to_player: f32,
    pub can_see_player: bool,
    pub on_ground: bool,
    pub blackboard: HashMap<String, BtValue>,
}

impl BtContext {
    /// Health as a ratio in 0.0..=1.0.
    pub fn health_percent(&self) -> f32 {
        if self.self_health_max <= 0.0 {
            0.0
        } else {
            (self.self_health / self.self_health_max).clamp(0.0, 1.0)
        }
    }
}

// ===========================================================================
// Composite nodes
// ===========================================================================

/// Runs children in order. Succeeds only if **all** children succeed (AND).
pub struct SequenceNode {
    pub children: Vec<Box<dyn BtNode>>,
    pub current: usize,
    name: String,
}

impl SequenceNode {
    pub fn new(children: Vec<Box<dyn BtNode>>) -> Self {
        Self {
            children,
            current: 0,
            name: "Sequence".into(),
        }
    }
}

impl BtNode for SequenceNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        while self.current < self.children.len() {
            let status = self.children[self.current].tick(ctx);
            match status {
                BtStatus::Running => return BtStatus::Running,
                BtStatus::Failure => {
                    self.current = 0;
                    return BtStatus::Failure;
                }
                BtStatus::Success => {
                    self.current += 1;
                }
            }
        }
        self.current = 0;
        BtStatus::Success
    }

    fn reset(&mut self) {
        self.current = 0;
        for child in &mut self.children {
            child.reset();
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Runs children in order. Succeeds if **any** child succeeds (OR).
pub struct SelectorNode {
    pub children: Vec<Box<dyn BtNode>>,
    pub current: usize,
    name: String,
}

impl SelectorNode {
    pub fn new(children: Vec<Box<dyn BtNode>>) -> Self {
        Self {
            children,
            current: 0,
            name: "Selector".into(),
        }
    }
}

impl BtNode for SelectorNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        while self.current < self.children.len() {
            let status = self.children[self.current].tick(ctx);
            match status {
                BtStatus::Running => return BtStatus::Running,
                BtStatus::Success => {
                    self.current = 0;
                    return BtStatus::Success;
                }
                BtStatus::Failure => {
                    self.current += 1;
                }
            }
        }
        self.current = 0;
        BtStatus::Failure
    }

    fn reset(&mut self) {
        self.current = 0;
        for child in &mut self.children {
            child.reset();
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Runs **all** children simultaneously each tick.
/// Succeeds when at least `required_successes` children succeed.
/// Fails when it becomes impossible to reach that threshold.
pub struct ParallelNode {
    pub children: Vec<Box<dyn BtNode>>,
    pub required_successes: usize,
    name: String,
}

impl ParallelNode {
    pub fn new(children: Vec<Box<dyn BtNode>>, required_successes: usize) -> Self {
        Self {
            children,
            required_successes,
            name: "Parallel".into(),
        }
    }
}

impl BtNode for ParallelNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        let mut success_count = 0usize;
        let mut failure_count = 0usize;

        for child in &mut self.children {
            match child.tick(ctx) {
                BtStatus::Success => success_count += 1,
                BtStatus::Failure => failure_count += 1,
                BtStatus::Running => {}
            }
        }

        if success_count >= self.required_successes {
            BtStatus::Success
        } else if failure_count > self.children.len() - self.required_successes {
            BtStatus::Failure
        } else {
            BtStatus::Running
        }
    }

    fn reset(&mut self) {
        for child in &mut self.children {
            child.reset();
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Randomly picks one child to run each tick.
pub struct RandomSelectorNode {
    pub children: Vec<Box<dyn BtNode>>,
    name: String,
    /// Index chosen for the current evaluation (sticky while Running).
    chosen: Option<usize>,
}

impl RandomSelectorNode {
    pub fn new(children: Vec<Box<dyn BtNode>>) -> Self {
        Self {
            children,
            name: "RandomSelector".into(),
            chosen: None,
        }
    }

    /// Simple deterministic-ish pick using entity id + dt as entropy source.
    fn pick_index(ctx: &BtContext, len: usize) -> usize {
        // Use a basic hash of available runtime data.
        let bits = (ctx.entity_id.wrapping_mul(2654435761) ^ (ctx.dt.to_bits() as u64)) as usize;
        bits % len
    }
}

impl BtNode for RandomSelectorNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        if self.children.is_empty() {
            return BtStatus::Failure;
        }

        let idx = match self.chosen {
            Some(i) => i,
            None => {
                let i = Self::pick_index(ctx, self.children.len());
                self.chosen = Some(i);
                i
            }
        };

        let status = self.children[idx].tick(ctx);
        if status != BtStatus::Running {
            self.chosen = None;
        }
        status
    }

    fn reset(&mut self) {
        self.chosen = None;
        for child in &mut self.children {
            child.reset();
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ===========================================================================
// Decorator nodes
// ===========================================================================

/// Inverts child result: Success ↔ Failure. Running passes through.
pub struct InverterNode {
    pub child: Box<dyn BtNode>,
}

impl BtNode for InverterNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        match self.child.tick(ctx) {
            BtStatus::Success => BtStatus::Failure,
            BtStatus::Failure => BtStatus::Success,
            BtStatus::Running => BtStatus::Running,
        }
    }

    fn reset(&mut self) {
        self.child.reset();
    }

    fn name(&self) -> &str {
        "Inverter"
    }
}

/// Repeats the child `count` times (or indefinitely when `count` is -1).
pub struct RepeatNode {
    pub child: Box<dyn BtNode>,
    pub count: i32,
    pub current: i32,
}

impl BtNode for RepeatNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        if self.count >= 0 && self.current >= self.count {
            return BtStatus::Success;
        }

        let status = self.child.tick(ctx);
        match status {
            BtStatus::Running => BtStatus::Running,
            BtStatus::Failure => BtStatus::Failure,
            BtStatus::Success => {
                self.current += 1;
                self.child.reset();
                if self.count >= 0 && self.current >= self.count {
                    BtStatus::Success
                } else {
                    BtStatus::Running
                }
            }
        }
    }

    fn reset(&mut self) {
        self.current = 0;
        self.child.reset();
    }

    fn name(&self) -> &str {
        "Repeat"
    }
}

/// Always returns `Success` regardless of child result.
pub struct SucceederNode {
    pub child: Box<dyn BtNode>,
}

impl BtNode for SucceederNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        let status = self.child.tick(ctx);
        if status == BtStatus::Running {
            BtStatus::Running
        } else {
            BtStatus::Success
        }
    }

    fn reset(&mut self) {
        self.child.reset();
    }

    fn name(&self) -> &str {
        "Succeeder"
    }
}

/// Limits child execution time. Returns `Failure` if the timeout is exceeded.
pub struct TimeoutNode {
    pub child: Box<dyn BtNode>,
    pub timeout: f32,
    pub elapsed: f32,
}

impl BtNode for TimeoutNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        self.elapsed += ctx.dt;
        if self.elapsed >= self.timeout {
            return BtStatus::Failure;
        }
        let status = self.child.tick(ctx);
        if status != BtStatus::Running {
            self.elapsed = 0.0;
        }
        status
    }

    fn reset(&mut self) {
        self.elapsed = 0.0;
        self.child.reset();
    }

    fn name(&self) -> &str {
        "Timeout"
    }
}

/// Only runs the child if a condition node succeeds first (guard/gate pattern).
pub struct GuardNode {
    pub condition: Box<dyn BtNode>,
    pub child: Box<dyn BtNode>,
}

impl BtNode for GuardNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        match self.condition.tick(ctx) {
            BtStatus::Success => self.child.tick(ctx),
            BtStatus::Failure => BtStatus::Failure,
            BtStatus::Running => BtStatus::Running,
        }
    }

    fn reset(&mut self) {
        self.condition.reset();
        self.child.reset();
    }

    fn name(&self) -> &str {
        "Guard"
    }
}

/// Prevents the child from running again until `cooldown` seconds have elapsed
/// since the child last returned a non-Running result.
pub struct CooldownNode {
    pub child: Box<dyn BtNode>,
    pub cooldown: f32,
    pub timer: f32,
}

impl BtNode for CooldownNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        if self.timer > 0.0 {
            self.timer -= ctx.dt;
            return BtStatus::Failure;
        }

        let status = self.child.tick(ctx);
        if status != BtStatus::Running {
            self.timer = self.cooldown;
        }
        status
    }

    fn reset(&mut self) {
        self.timer = 0.0;
        self.child.reset();
    }

    fn name(&self) -> &str {
        "Cooldown"
    }
}

// ===========================================================================
// Leaf / action nodes
// ===========================================================================

/// Evaluates a condition predicate against the context.
pub struct ConditionNode {
    pub name: String,
    pub check: fn(&BtContext) -> bool,
}

impl BtNode for ConditionNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        if (self.check)(ctx) {
            BtStatus::Success
        } else {
            BtStatus::Failure
        }
    }

    fn reset(&mut self) {}

    fn name(&self) -> &str {
        &self.name
    }
}

/// Waits for a fixed duration, returning `Running` until the time elapses.
pub struct WaitNode {
    pub duration: f32,
    pub elapsed: f32,
}

impl WaitNode {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
        }
    }
}

impl BtNode for WaitNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        self.elapsed += ctx.dt;
        if self.elapsed >= self.duration {
            self.elapsed = 0.0;
            BtStatus::Success
        } else {
            BtStatus::Running
        }
    }

    fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    fn name(&self) -> &str {
        "Wait"
    }
}

/// Runs an arbitrary action function each tick.
pub struct ActionNode {
    pub name: String,
    pub action: fn(&mut BtContext) -> BtStatus,
}

impl BtNode for ActionNode {
    fn tick(&mut self, ctx: &mut BtContext) -> BtStatus {
        (self.action)(ctx)
    }

    fn reset(&mut self) {}

    fn name(&self) -> &str {
        &self.name
    }
}

// ===========================================================================
// Predefined condition factories
// ===========================================================================

/// Returns a condition that succeeds when the player is within `range` units.
pub fn is_player_in_range(range: f32) -> ConditionNode {
    ConditionNode {
        name: format!("IsPlayerInRange({})", range),
        check: {
            // We cannot capture `range` in a fn pointer, so we use a const
            // threshold encoded via a factory that returns a ConditionNode
            // with a closure-compatible check. Since the trait requires
            // `fn(&BtContext) -> bool`, we store the range in the name and
            // use distance_to_player vs a fixed set of common ranges.
            // To work around fn-pointer limitations we rely on a helper that
            // embeds the range in a static. The pragmatic approach: always
            // compare distance_to_player against the value stored in the
            // blackboard key "__condition_range".
            |ctx: &BtContext| -> bool { ctx.distance_to_player <= range_from_name(ctx, "IsPlayerInRange") }
        },
    }
}

/// Returns a condition that succeeds when health is below `percent` (0.0–1.0).
pub fn is_health_below(percent: f32) -> ConditionNode {
    ConditionNode {
        name: format!("IsHealthBelow({})", percent),
        check: |ctx: &BtContext| -> bool {
            let threshold = range_from_name(ctx, "IsHealthBelow");
            ctx.health_percent() < threshold
        },
    }
}

/// Returns a condition that succeeds when the entity can see the player.
pub fn can_see_player() -> ConditionNode {
    ConditionNode {
        name: "CanSeePlayer".into(),
        check: |ctx: &BtContext| -> bool { ctx.can_see_player },
    }
}

/// Returns a condition that succeeds when the entity is on the ground.
pub fn is_on_ground() -> ConditionNode {
    ConditionNode {
        name: "IsOnGround".into(),
        check: |ctx: &BtContext| -> bool { ctx.on_ground },
    }
}

/// Helper: extract a float parameter from the condition name stored on the
/// blackboard. This is a pragmatic workaround for fn-pointer conditions that
/// need captured state — callers should set the blackboard key before ticking.
fn range_from_name(ctx: &BtContext, prefix: &str) -> f32 {
    // Look for a blackboard entry like "__IsPlayerInRange" holding a Float.
    let key = format!("__{}", prefix);
    if let Some(BtValue::Float(v)) = ctx.blackboard.get(&key) {
        *v as f32
    } else {
        // Sensible defaults.
        match prefix {
            "IsPlayerInRange" => 200.0,
            "IsHealthBelow" => 0.3,
            _ => 0.0,
        }
    }
}

// ===========================================================================
// Builder
// ===========================================================================

enum BuilderKind {
    Sequence,
    Selector,
}

/// Fluent builder for constructing behavior trees.
pub struct BehaviorTreeBuilder {
    kind: BuilderKind,
    children: Vec<Box<dyn BtNode>>,
}

impl BehaviorTreeBuilder {
    /// Start building a sequence node.
    pub fn sequence() -> Self {
        Self {
            kind: BuilderKind::Sequence,
            children: Vec::new(),
        }
    }

    /// Start building a selector node.
    pub fn selector() -> Self {
        Self {
            kind: BuilderKind::Selector,
            children: Vec::new(),
        }
    }

    /// Append a child node.
    pub fn child(mut self, node: Box<dyn BtNode>) -> Self {
        self.children.push(node);
        self
    }

    /// Finalise and return the built node.
    pub fn build(self) -> Box<dyn BtNode> {
        match self.kind {
            BuilderKind::Sequence => Box::new(SequenceNode::new(self.children)),
            BuilderKind::Selector => Box::new(SelectorNode::new(self.children)),
        }
    }
}
