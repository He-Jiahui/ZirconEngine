use crate::core::state::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};

use super::CoreHandle;

impl CoreHandle {
    pub fn init_state<T>(&self) -> StateTransitionEvent<T>
    where
        T: StateSpec + Default,
    {
        let dispatch = {
            self.inner
                .states
                .lock()
                .unwrap()
                .init_state::<T>(T::default())
        };
        if let Some(dispatch) = dispatch {
            let event = dispatch.event().clone();
            dispatch.run();
            return event;
        }

        StateTransitionEvent::new(None, self.state::<T>().map(State::into_inner), true)
    }

    pub fn insert_state<T: StateSpec>(&self, state: T) -> StateTransitionEvent<T> {
        let dispatch = self.inner.states.lock().unwrap().insert_state(state);
        let event = dispatch.event().clone();
        dispatch.run();
        event
    }

    pub fn state<T: StateSpec>(&self) -> Option<State<T>> {
        self.inner.states.lock().unwrap().state::<T>()
    }

    pub fn next_state<T: StateSpec>(&self) -> NextState<T> {
        self.inner.states.lock().unwrap().next_state::<T>()
    }

    pub fn set_next_state<T: StateSpec>(&self, state: T) {
        self.inner.states.lock().unwrap().set_next_state(state);
    }

    pub fn set_next_state_if_neq<T: StateSpec>(&self, state: T) {
        self.inner
            .states
            .lock()
            .unwrap()
            .set_next_state_if_neq(state);
    }

    pub fn reset_next_state<T: StateSpec>(&self) {
        self.inner.states.lock().unwrap().reset_next_state::<T>();
    }

    pub fn apply_state_transition<T: StateSpec>(&self) -> Option<StateTransitionEvent<T>> {
        let dispatch = self
            .inner
            .states
            .lock()
            .unwrap()
            .apply_state_transition::<T>()?;
        let event = dispatch.event().clone();
        dispatch.run();
        Some(event)
    }

    pub fn state_transition_events<T: StateSpec>(&self) -> Vec<StateTransitionEvent<T>> {
        self.inner.states.lock().unwrap().transition_events::<T>()
    }

    pub fn register_on_enter<T, F>(&self, label: OnEnter<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.inner
            .states
            .lock()
            .unwrap()
            .register_on_enter(label, hook);
    }

    pub fn register_on_exit<T, F>(&self, label: OnExit<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.inner
            .states
            .lock()
            .unwrap()
            .register_on_exit(label, hook);
    }

    pub fn register_on_transition<T, F>(&self, label: OnTransition<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.inner
            .states
            .lock()
            .unwrap()
            .register_on_transition(label, hook);
    }
}
