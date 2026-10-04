use std::sync::MutexGuard;

use crate::core::runtime::state_machine::{
    NextState, OnEnter, OnExit, OnTransition, State, StateRegistry, StateSpec, StateTransitionEvent,
};

use super::CoreHandle;

impl CoreHandle {
    /// 首次创建状态时派发进入事件；已有状态复用当前值且不会重复执行进入回调。
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

    /// 在状态提交点应用待定转换；回调在释放注册表锁后执行，可再次访问状态。
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
#[path = "tests/states.rs"]
mod tests;
