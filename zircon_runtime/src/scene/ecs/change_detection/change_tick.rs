use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChangeTick(u64);

impl ChangeTick {
    pub const ZERO: Self = Self(0);
    pub const INITIAL: Self = Self(1);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub fn is_newer_than(self, last_run: Self, this_run: Self) -> bool {
        self.0 > last_run.0 && self.0 <= this_run.0
    }
}
