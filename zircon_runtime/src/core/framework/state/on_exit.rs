use super::StateSpec;

/// Hook label that runs when a state machine exits the matching value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnExit<T: StateSpec> {
    pub state: T,
}

impl<T: StateSpec> OnExit<T> {
    pub fn new(state: T) -> Self {
        Self { state }
    }
}
