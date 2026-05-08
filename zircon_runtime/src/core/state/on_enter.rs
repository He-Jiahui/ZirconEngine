use super::StateSpec;

/// Hook label that runs when a state machine enters the matching value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnEnter<T: StateSpec> {
    pub state: T,
}

impl<T: StateSpec> OnEnter<T> {
    pub fn new(state: T) -> Self {
        Self { state }
    }
}
