use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreferenceWorkDeadline {
    deadline: Option<Instant>,
}

impl PreferenceWorkDeadline {
    pub const fn none() -> Self {
        Self { deadline: None }
    }

    pub const fn at(deadline: Instant) -> Self {
        Self {
            deadline: Some(deadline),
        }
    }

    pub const fn instant(self) -> Option<Instant> {
        self.deadline
    }
}
