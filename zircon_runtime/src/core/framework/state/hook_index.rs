use std::collections::HashMap;
use std::sync::Arc;

use super::{
    hook::{StateHook, StateTransitionDispatch},
    OnEnter, OnExit, OnTransition, StateSpec, StateTransitionEvent,
};

/// Canonical hash-bucket owner for state transition hooks.
pub(crate) struct StateHookIndex<T: StateSpec> {
    on_enter: HashMap<T, Vec<StateHook<T>>>,
    on_exit: HashMap<T, Vec<StateHook<T>>>,
    on_transition: HashMap<T, HashMap<T, Vec<StateHook<T>>>>,
}

impl<T: StateSpec> Default for StateHookIndex<T> {
    fn default() -> Self {
        Self {
            on_enter: HashMap::new(),
            on_exit: HashMap::new(),
            on_transition: HashMap::new(),
        }
    }
}

impl<T: StateSpec> StateHookIndex<T> {
    pub(crate) fn register_on_enter<F>(&mut self, label: OnEnter<T>, hook: F)
    where
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.on_enter
            .entry(label.state)
            .or_default()
            .push(Arc::new(hook));
    }

    pub(crate) fn register_on_exit<F>(&mut self, label: OnExit<T>, hook: F)
    where
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.on_exit
            .entry(label.state)
            .or_default()
            .push(Arc::new(hook));
    }

    pub(crate) fn register_on_transition<F>(&mut self, label: OnTransition<T>, hook: F)
    where
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.on_transition
            .entry(label.exited)
            .or_default()
            .entry(label.entered)
            .or_default()
            .push(Arc::new(hook));
    }

    pub(crate) fn dispatch(&self, event: StateTransitionEvent<T>) -> StateTransitionDispatch<T> {
        let exit_hooks = self.exit_hooks_for(&event);
        let transition_hooks = self.transition_hooks_for(&event);
        let enter_hooks = self.enter_hooks_for(&event);
        StateTransitionDispatch::new(event, exit_hooks, transition_hooks, enter_hooks)
    }

    fn enter_hooks_for(&self, event: &StateTransitionEvent<T>) -> Vec<StateHook<T>> {
        let Some(entered) = event.entered.as_ref() else {
            return Vec::new();
        };
        self.on_enter.get(entered).cloned().unwrap_or_default()
    }

    fn exit_hooks_for(&self, event: &StateTransitionEvent<T>) -> Vec<StateHook<T>> {
        let Some(exited) = event.exited.as_ref() else {
            return Vec::new();
        };
        self.on_exit.get(exited).cloned().unwrap_or_default()
    }

    fn transition_hooks_for(&self, event: &StateTransitionEvent<T>) -> Vec<StateHook<T>> {
        let (Some(exited), Some(entered)) = (event.exited.as_ref(), event.entered.as_ref()) else {
            return Vec::new();
        };
        self.on_transition
            .get(exited)
            .and_then(|targets| targets.get(entered))
            .cloned()
            .unwrap_or_default()
    }
}
