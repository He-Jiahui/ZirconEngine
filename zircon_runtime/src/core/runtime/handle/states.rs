use std::sync::MutexGuard;

use crate::core::runtime::state_machine::{
    NextState, OnEnter, OnExit, OnTransition, State, StateRegistry, StateSpec, StateTransitionEvent,
};

use super::CoreHandle;

impl CoreHandle {
    pub fn init_state<T>(&self) -> StateTransitionEvent<T>
    where
        T: StateSpec + Default,
    {
        let (dispatch, entered) = {
            let mut states = self.lock_states();
            let dispatch = states.init_state::<T>(T::default());
            let entered = if dispatch.is_none() {
                states.state::<T>().map(State::into_inner)
            } else {
                None
            };
            (dispatch, entered)
        };
        if let Some(dispatch) = dispatch {
            let event = dispatch.event().clone();
            dispatch.run();
            return event;
        }

        StateTransitionEvent::new(None, entered, true)
    }

    pub fn insert_state<T: StateSpec>(&self, state: T) -> StateTransitionEvent<T> {
        let dispatch = self.lock_states().insert_state(state);
        let event = dispatch.event().clone();
        dispatch.run();
        event
    }

    pub fn state<T: StateSpec>(&self) -> Option<State<T>> {
        self.lock_states().state::<T>()
    }

    pub fn next_state<T: StateSpec>(&self) -> NextState<T> {
        self.lock_states().next_state::<T>()
    }

    pub fn set_next_state<T: StateSpec>(&self, state: T) {
        self.lock_states().set_next_state(state);
    }

    pub fn set_next_state_if_neq<T: StateSpec>(&self, state: T) {
        self.lock_states().set_next_state_if_neq(state);
    }

    pub fn reset_next_state<T: StateSpec>(&self) {
        self.lock_states().reset_next_state::<T>();
    }

    pub fn apply_state_transition<T: StateSpec>(&self) -> Option<StateTransitionEvent<T>> {
        let dispatch = self.lock_states().apply_state_transition::<T>()?;
        let event = dispatch.event().clone();
        dispatch.run();
        Some(event)
    }

    pub fn latest_state_transition<T: StateSpec>(&self) -> Option<StateTransitionEvent<T>> {
        self.lock_states().latest_transition::<T>()
    }

    pub fn register_on_enter<T, F>(&self, label: OnEnter<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.lock_states().register_on_enter(label, hook);
    }

    pub fn register_on_exit<T, F>(&self, label: OnExit<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.lock_states().register_on_exit(label, hook);
    }

    pub fn register_on_transition<T, F>(&self, label: OnTransition<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.lock_states().register_on_transition(label, hook);
    }

    fn lock_states(&self) -> MutexGuard<'_, StateRegistry> {
        self.inner
            .states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use crate::core::runtime::state_machine::NextState;
    use crate::core::CoreRuntime;

    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
    enum StateFixture {
        #[default]
        Boot,
        Running,
    }

    #[test]
    fn core_handle_state_accessors_recover_poisoned_state_registry_lock() {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.states.lock().unwrap();
            panic!("poison core handle state registry");
        }));

        let init_event = handle.init_state::<StateFixture>();
        assert_eq!(init_event.entered, Some(StateFixture::Boot));
        assert_eq!(
            handle.state::<StateFixture>().unwrap().into_inner(),
            StateFixture::Boot
        );

        handle.set_next_state(StateFixture::Running);
        assert_eq!(
            handle.next_state::<StateFixture>(),
            NextState::Pending(StateFixture::Running)
        );

        let transition = handle.apply_state_transition::<StateFixture>().unwrap();
        assert_eq!(transition.exited, Some(StateFixture::Boot));
        assert_eq!(transition.entered, Some(StateFixture::Running));
        assert_eq!(
            handle.state::<StateFixture>().unwrap().into_inner(),
            StateFixture::Running
        );

        let latest = handle.latest_state_transition::<StateFixture>();
        assert_eq!(latest, Some(transition));

        handle.reset_next_state::<StateFixture>();
        assert_eq!(handle.next_state::<StateFixture>(), NextState::Unchanged);
    }

    #[test]
    fn existing_state_init_reuses_the_registry_lock() {
        let source = include_str!("states.rs");
        let start = source.find("pub fn init_state").expect("init state method");
        let end = source[start..]
            .find("pub fn insert_state")
            .map(|offset| start + offset)
            .expect("insert state method");
        let implementation = &source[start..end];

        assert!(implementation.contains("let mut states = self.lock_states();"));
        assert!(implementation.contains("states.state::<T>()"));
        assert!(!implementation.contains("self.state::<T>()"));
    }
}
