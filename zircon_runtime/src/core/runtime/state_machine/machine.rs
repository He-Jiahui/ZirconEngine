use super::{
    hook::StateTransitionDispatch, NextState, OnEnter, OnExit, OnTransition, State, StateHookIndex,
    StateSpec, StateTransitionEvent,
};

pub(crate) struct StateMachine<T: StateSpec> {
    current: Option<State<T>>,
    next: NextState<T>,
    latest_event: Option<StateTransitionEvent<T>>,
    hooks: StateHookIndex<T>,
}

impl<T: StateSpec> Default for StateMachine<T> {
    fn default() -> Self {
        Self {
            current: None,
            next: NextState::Unchanged,
            latest_event: None,
            hooks: StateHookIndex::default(),
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

    pub(crate) fn latest_event(&self) -> Option<StateTransitionEvent<T>> {
        self.latest_event.clone()
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
        self.hooks.register_on_enter(label, hook);
    }

    pub(crate) fn register_on_exit<F>(&mut self, label: OnExit<T>, hook: F)
    where
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.hooks.register_on_exit(label, hook);
    }

    pub(crate) fn register_on_transition<F>(&mut self, label: OnTransition<T>, hook: F)
    where
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.hooks.register_on_transition(label, hook);
    }

    fn record_transition(
        &mut self,
        exited: Option<T>,
        entered: Option<T>,
        allow_same_state_transitions: bool,
    ) -> StateTransitionDispatch<T> {
        let event = StateTransitionEvent::new(exited, entered, allow_same_state_transitions);
        self.latest_event = Some(event.clone());
        self.hooks.dispatch(event)
    }
}
