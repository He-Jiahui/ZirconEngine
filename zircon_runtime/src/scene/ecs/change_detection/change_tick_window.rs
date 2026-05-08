use super::ChangeTick;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeTickWindow {
    last_run: ChangeTick,
    this_run: ChangeTick,
}

impl ChangeTickWindow {
    pub const fn new(last_run: ChangeTick, this_run: ChangeTick) -> Self {
        Self { last_run, this_run }
    }

    pub const fn all(this_run: ChangeTick) -> Self {
        Self {
            last_run: ChangeTick::ZERO,
            this_run,
        }
    }

    pub const fn last_run(self) -> ChangeTick {
        self.last_run
    }

    pub const fn this_run(self) -> ChangeTick {
        self.this_run
    }
}
