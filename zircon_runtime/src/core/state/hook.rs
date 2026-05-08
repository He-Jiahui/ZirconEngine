use std::sync::Arc;

use super::{StateSpec, StateTransitionEvent};

pub(crate) type StateHook<T> = Arc<dyn Fn(&StateTransitionEvent<T>) + Send + Sync + 'static>;

/// Deferred hook invocation bundle for one state transition.
pub(crate) struct StateTransitionDispatch<T: StateSpec> {
    event: StateTransitionEvent<T>,
    exit_hooks: Vec<StateHook<T>>,
    transition_hooks: Vec<StateHook<T>>,
    enter_hooks: Vec<StateHook<T>>,
}

impl<T: StateSpec> StateTransitionDispatch<T> {
    pub(crate) fn new(
        event: StateTransitionEvent<T>,
        exit_hooks: Vec<StateHook<T>>,
        transition_hooks: Vec<StateHook<T>>,
        enter_hooks: Vec<StateHook<T>>,
    ) -> Self {
        Self {
            event,
            exit_hooks,
            transition_hooks,
            enter_hooks,
        }
    }

    pub(crate) fn event(&self) -> &StateTransitionEvent<T> {
        &self.event
    }

    pub(crate) fn run(self) {
        for hook in self.exit_hooks {
            hook(&self.event);
        }
        for hook in self.transition_hooks {
            hook(&self.event);
        }
        for hook in self.enter_hooks {
            hook(&self.event);
        }
    }
}
