use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::{
    hook::StateTransitionDispatch, machine::StateMachine, NextState, OnEnter, OnExit, OnTransition,
    State, StateSpec, StateTransitionEvent,
};

#[derive(Default)]
pub(crate) struct StateRegistry {
    machines: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl StateRegistry {
    pub(crate) fn init_state<T: StateSpec>(
        &mut self,
        initial: T,
    ) -> Option<StateTransitionDispatch<T>> {
        self.machine_or_insert_mut::<T>().init(initial)
    }

    pub(crate) fn insert_state<T: StateSpec>(&mut self, state: T) -> StateTransitionDispatch<T> {
        self.machine_or_insert_mut::<T>().insert(state)
    }

    pub(crate) fn state<T: StateSpec>(&self) -> Option<State<T>> {
        self.machine::<T>().and_then(StateMachine::current)
    }

    pub(crate) fn next_state<T: StateSpec>(&self) -> NextState<T> {
        self.machine::<T>()
            .map(StateMachine::next)
            .unwrap_or_default()
    }

    pub(crate) fn set_next_state<T: StateSpec>(&mut self, state: T) {
        self.machine_or_insert_mut::<T>().set_next(state);
    }

    pub(crate) fn set_next_state_if_neq<T: StateSpec>(&mut self, state: T) {
        self.machine_or_insert_mut::<T>().set_next_if_neq(state);
    }

    pub(crate) fn reset_next_state<T: StateSpec>(&mut self) {
        if let Some(machine) = self.machine_mut::<T>() {
            machine.reset_next();
        }
    }

    pub(crate) fn apply_state_transition<T: StateSpec>(
        &mut self,
    ) -> Option<StateTransitionDispatch<T>> {
        self.machine_mut::<T>()?.apply_pending_transition()
    }

    pub(crate) fn transition_events<T: StateSpec>(&self) -> Vec<StateTransitionEvent<T>> {
        self.machine::<T>()
            .map(StateMachine::events)
            .unwrap_or_default()
    }

    pub(crate) fn register_on_enter<T, F>(&mut self, label: OnEnter<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.machine_or_insert_mut::<T>()
            .register_on_enter(label, hook);
    }

    pub(crate) fn register_on_exit<T, F>(&mut self, label: OnExit<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.machine_or_insert_mut::<T>()
            .register_on_exit(label, hook);
    }

    pub(crate) fn register_on_transition<T, F>(&mut self, label: OnTransition<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.machine_or_insert_mut::<T>()
            .register_on_transition(label, hook);
    }

    fn machine<T: StateSpec>(&self) -> Option<&StateMachine<T>> {
        self.machines
            .get(&TypeId::of::<T>())
            .and_then(|machine| machine.downcast_ref::<StateMachine<T>>())
    }

    fn machine_mut<T: StateSpec>(&mut self) -> Option<&mut StateMachine<T>> {
        self.machines
            .get_mut(&TypeId::of::<T>())
            .and_then(|machine| machine.downcast_mut::<StateMachine<T>>())
    }

    fn machine_or_insert_mut<T: StateSpec>(&mut self) -> &mut StateMachine<T> {
        self.machines
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<StateMachine<T>>::default());
        self.machine_mut::<T>()
            .expect("state registry stored a machine under the wrong TypeId")
    }
}
