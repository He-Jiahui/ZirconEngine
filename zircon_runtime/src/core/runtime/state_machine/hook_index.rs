use std::collections::HashMap;
use std::sync::Arc;

use super::{
    hook::{StateHook, StateTransitionDispatch},
    OnEnter, OnExit, OnTransition, StateSpec, StateTransitionEvent,
};

/// 按状态标签保留注册顺序；dispatch 依次合并 exit、transition、enter 钩子。
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
        let mut ordered_hooks =
            Vec::with_capacity(exit_hooks.len() + transition_hooks.len() + enter_hooks.len());
        ordered_hooks.extend(exit_hooks.iter().cloned());
        ordered_hooks.extend(transition_hooks.iter().cloned());
        ordered_hooks.extend(enter_hooks.iter().cloned());
        StateTransitionDispatch::new(event, ordered_hooks)
    }

    fn enter_hooks_for(&self, event: &StateTransitionEvent<T>) -> &[StateHook<T>] {
        let Some(entered) = event.entered.as_ref() else {
            return &[];
        };
        self.on_enter.get(entered).map(Vec::as_slice).unwrap_or(&[])
    }

    fn exit_hooks_for(&self, event: &StateTransitionEvent<T>) -> &[StateHook<T>] {
        let Some(exited) = event.exited.as_ref() else {
            return &[];
        };
        self.on_exit.get(exited).map(Vec::as_slice).unwrap_or(&[])
    }

    fn transition_hooks_for(&self, event: &StateTransitionEvent<T>) -> &[StateHook<T>] {
        let (Some(exited), Some(entered)) = (event.exited.as_ref(), event.entered.as_ref()) else {
            return &[];
        };
        self.on_transition
            .get(exited)
            .and_then(|targets| targets.get(entered))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

#[cfg(test)]
#[path = "hook_index/tests/optimization_batch_hu_runtime603_tests.rs"]
mod optimization_batch_hu_runtime603_tests;
