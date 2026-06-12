use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChangeTick(u64);

impl ChangeTick {
    pub const ZERO: Self = Self(0);
    pub const INITIAL: Self = Self(1);
    pub const CHECK_TICK_THRESHOLD: u64 = 518_400_000;
    pub const MAX_CHANGE_AGE: u64 = u64::MAX - (2 * Self::CHECK_TICK_THRESHOLD - 1);
    pub const MAX: Self = Self(Self::MAX_CHANGE_AGE);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub const fn relative_to(self, older: Self) -> Self {
        Self(self.0.wrapping_sub(older.0))
    }

    pub(crate) const fn clamp_older_than(self, present: Self) -> Self {
        let age = present.relative_to(self).0;
        if age > Self::MAX_CHANGE_AGE {
            present.relative_to(Self::MAX)
        } else {
            self
        }
    }

    pub fn is_newer_than(self, last_run: Self, this_run: Self) -> bool {
        let ticks_since_change = this_run.relative_to(self).0.min(Self::MAX_CHANGE_AGE);
        let ticks_since_system = this_run.relative_to(last_run).0.min(Self::MAX_CHANGE_AGE);

        ticks_since_system > ticks_since_change
    }
}
