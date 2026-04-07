//! Generic finite state machine framework for AI behaviors.

use rustc_hash::FxHashMap;
use std::fmt;
use std::hash::Hash;

/// A condition function that determines whether a state transition should fire.
pub type TransitionCondition<S, C> = Box<dyn Fn(&S, &C) -> bool + Send + Sync>;

/// An action function executed when entering/exiting a state or during a state update.
pub type StateAction<S, C> = Box<dyn FnMut(&mut S, &mut C, f32) + Send + Sync>;

/// A transition from one state to another with a guard condition.
struct Transition<S: Copy + Eq + Hash, StateData, Ctx> {
    from: S,
    to: S,
    condition: TransitionCondition<StateData, Ctx>,
}

/// A generic finite state machine parameterized by state enum, state data, and context.
pub struct StateMachine<S: Copy + Eq + Hash + fmt::Debug, StateData, Ctx> {
    current_state: S,
    transitions: Vec<Transition<S, StateData, Ctx>>,
    on_enter: FxHashMap<S, Vec<StateAction<StateData, Ctx>>>,
    on_update: FxHashMap<S, Vec<StateAction<StateData, Ctx>>>,
    on_exit: FxHashMap<S, Vec<StateAction<StateData, Ctx>>>,
    state_time: f32,
}

impl<S: Copy + Eq + Hash + fmt::Debug, StateData, Ctx> StateMachine<S, StateData, Ctx> {
    /// Create a new state machine with the given initial state.
    pub fn new(initial_state: S) -> Self {
        Self {
            current_state: initial_state,
            transitions: Vec::new(),
            on_enter: FxHashMap::default(),
            on_update: FxHashMap::default(),
            on_exit: FxHashMap::default(),
            state_time: 0.0,
        }
    }

    /// Add a guarded transition from one state to another.
    pub fn add_transition(
        &mut self,
        from: S,
        to: S,
        condition: impl Fn(&StateData, &Ctx) -> bool + Send + Sync + 'static,
    ) {
        self.transitions.push(Transition {
            from,
            to,
            condition: Box::new(condition),
        });
    }

    /// Register an action to run when entering a state.
    pub fn on_enter(
        &mut self,
        state: S,
        action: impl FnMut(&mut StateData, &mut Ctx, f32) + Send + Sync + 'static,
    ) {
        self.on_enter
            .entry(state)
            .or_default()
            .push(Box::new(action));
    }

    /// Register an action to run each frame while in a state.
    pub fn on_update(
        &mut self,
        state: S,
        action: impl FnMut(&mut StateData, &mut Ctx, f32) + Send + Sync + 'static,
    ) {
        self.on_update
            .entry(state)
            .or_default()
            .push(Box::new(action));
    }

    /// Register an action to run when leaving a state.
    pub fn on_exit(
        &mut self,
        state: S,
        action: impl FnMut(&mut StateData, &mut Ctx, f32) + Send + Sync + 'static,
    ) {
        self.on_exit
            .entry(state)
            .or_default()
            .push(Box::new(action));
    }

    /// Get the current state.
    pub fn current_state(&self) -> S {
        self.current_state
    }

    /// Get the time spent in the current state.
    pub fn state_time(&self) -> f32 {
        self.state_time
    }

    /// Force a state transition (bypassing conditions), running exit/enter actions.
    pub fn force_transition(&mut self, new_state: S, data: &mut StateData, ctx: &mut Ctx, dt: f32) {
        // Run exit actions for old state
        if let Some(actions) = self.on_exit.get_mut(&self.current_state) {
            for action in actions.iter_mut() {
                action(data, ctx, dt);
            }
        }

        self.current_state = new_state;
        self.state_time = 0.0;

        // Run enter actions for new state
        if let Some(actions) = self.on_enter.get_mut(&self.current_state) {
            for action in actions.iter_mut() {
                action(data, ctx, dt);
            }
        }
    }

    /// Evaluate transitions and run update actions. Call each frame.
    pub fn update(&mut self, data: &mut StateData, ctx: &mut Ctx, dt: f32) {
        self.state_time += dt;

        // Check transitions
        let mut next_state = None;
        for transition in &self.transitions {
            if transition.from == self.current_state && (transition.condition)(data, ctx) {
                next_state = Some(transition.to);
                break;
            }
        }

        // Perform transition if one was triggered
        if let Some(new_state) = next_state {
            self.force_transition(new_state, data, ctx, dt);
        }

        // Run update actions for current state
        if let Some(actions) = self.on_update.get_mut(&self.current_state) {
            for action in actions.iter_mut() {
                action(data, ctx, dt);
            }
        }
    }
}
