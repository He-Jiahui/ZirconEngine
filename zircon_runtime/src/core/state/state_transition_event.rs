use super::StateSpec;

/// Event emitted whenever a runtime-wide state transition is applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateTransitionEvent<T: StateSpec> {
    pub exited: Option<T>,
    pub entered: Option<T>,
    pub allow_same_state_transitions: bool,
}

impl<T: StateSpec> StateTransitionEvent<T> {
    pub fn new(exited: Option<T>, entered: Option<T>, allow_same_state_transitions: bool) -> Self {
        Self {
            exited,
            entered,
            allow_same_state_transitions,
        }
    }
}
