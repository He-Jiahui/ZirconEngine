use std::ops::Deref;

use super::StateSpec;

/// Current value for one runtime-wide state machine.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct State<T: StateSpec>(T);

impl<T: StateSpec> State<T> {
    pub fn new(state: T) -> Self {
        Self(state)
    }

    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: StateSpec> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T: StateSpec> From<T> for State<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
