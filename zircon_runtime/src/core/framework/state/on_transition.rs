use super::StateSpec;

/// Hook label that runs between matching exited and entered state values.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnTransition<T: StateSpec> {
    pub exited: T,
    pub entered: T,
}

impl<T: StateSpec> OnTransition<T> {
    pub fn new(exited: T, entered: T) -> Self {
        Self { exited, entered }
    }
}
