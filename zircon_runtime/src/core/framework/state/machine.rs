use std::sync::Arc;

use super::{
    hook::{StateHook, StateTransitionDispatch},
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};

pub(crate) struct StateMachine<T: StateSpec> {
    current: Option<State<T>>,
    next: NextState<T>,
    events: Vec<StateTransitionEvent<T>>,
    on_enter: Vec<(OnEnter<T>, StateHook<T>)>,
    on_exit: Vec<(OnExit<T>, StateHook<T>)>,
    on_transition: Vec<(OnTransition<T>, StateHook<T>)>,
}

impl<T: StateSpec> Default for StateMachine<T> {
    fn default() -> Self {
        Self {
            current: None,
            next: NextState::Unchanged,
            events: Vec::new(),
            on_enter: Vec::new(),
            on_exit: Vec::new(),
            on_transition: Vec::new(),
        }
    }
}

impl<T: StateSpec> StateMachine<T> {
    pub(crate) fn current(&self) -> Option<State<T>> {
        self.current.clone()
    }

    pub(crate) fn next(&self) -> NextState<T> {
        self.next.clone()
    }

    pub(crate) fn events(&self) -> Vec<StateTransitionEvent<T>> {
        self.events.clone()
    }

    pub(crate) fn init(&mut self, initial: T) -> Option<StateTransitionDispatch<T>> {
        if self.current.is_some() {
            return None;
        }
        self.current = Some(State::new(initial.clone()));
        self.next.reset();
        Some(self.record_transition(None, Some(initial), true))
    }

    pub(crate) fn insert(&mut self, state: T) -> StateTransitionDispatch<T> {
        self.current = Some(State::new(state.clone()));
        self.next.reset();
        self.record_transition(None, Some(state), true)
    }

    pub(crate) fn set_next(&mut self, state: T) {
        self.next.set(state);
    }

    pub(crate) fn set_next_if_neq(&mut self, state: T) {
        self.next.set_if_neq(state);
    }

    pub(crate) fn reset_next(&mut self) {
        self.next.reset();
    }

    pub(crate) fn apply_pending_transition(&mut self) -> Option<StateTransitionDispatch<T>> {
        let (entered, allow_same_state_transitions) = self.next.take_transition()?;
        let exited = self.current.as_ref().map(|state| state.get().clone());

        if exited.as_ref() == Some(&entered) && !allow_same_state_transitions {
            return None;
        }

        self.current = Some(State::new(entered.clone()));
        Some(self.record_transition(exited, Some(entered), allow_same_state_transitions))
    }

    pub(crate) fn register_on_enter<F>(&mut self, label: OnEnter<T>, hook: F)
    where
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.on_enter.push((label, Arc::new(hook)));
    }

    pub(crate) fn register_on_exit<F>(&mut self, label: OnExit<T>, hook: F)
    where
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.on_exit.push((label, Arc::new(hook)));
    }

    pub(crate) fn register_on_transition<F>(&mut self, label: OnTransition<T>, hook: F)
    where
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.on_transition.push((label, Arc::new(hook)));
    }

    fn record_transition(
        &mut self,
        exited: Option<T>,
        entered: Option<T>,
        allow_same_state_transitions: bool,
    ) -> StateTransitionDispatch<T> {
        let event = StateTransitionEvent::new(exited, entered, allow_same_state_transitions);
        self.events.push(event.clone());
        let exit_hooks = self.exit_hooks_for(&event);
        let transition_hooks = self.transition_hooks_for(&event);
        let enter_hooks = self.enter_hooks_for(&event);
        StateTransitionDispatch::new(event, exit_hooks, transition_hooks, enter_hooks)
    }

    fn enter_hooks_for(&self, event: &StateTransitionEvent<T>) -> Vec<StateHook<T>> {
        let Some(entered) = event.entered.as_ref() else {
            return Vec::new();
        };
        self.on_enter
            .iter()
            .filter(|(label, _)| &label.state == entered)
            .map(|(_, hook)| Arc::clone(hook))
            .collect()
    }

    fn exit_hooks_for(&self, event: &StateTransitionEvent<T>) -> Vec<StateHook<T>> {
        let Some(exited) = event.exited.as_ref() else {
            return Vec::new();
        };
        self.on_exit
            .iter()
            .filter(|(label, _)| &label.state == exited)
            .map(|(_, hook)| Arc::clone(hook))
            .collect()
    }

    fn transition_hooks_for(&self, event: &StateTransitionEvent<T>) -> Vec<StateHook<T>> {
        let (Some(exited), Some(entered)) = (event.exited.as_ref(), event.entered.as_ref()) else {
            return Vec::new();
        };
        self.on_transition
            .iter()
            .filter(|(label, _)| &label.exited == exited && &label.entered == entered)
            .map(|(_, hook)| Arc::clone(hook))
            .collect()
    }
}
