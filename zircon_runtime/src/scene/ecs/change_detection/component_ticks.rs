use super::{ChangeTick, ChangeTickWindow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentTicks {
    added: ChangeTick,
    changed: ChangeTick,
}

impl ComponentTicks {
    pub const fn new(tick: ChangeTick) -> Self {
        Self {
            added: tick,
            changed: tick,
        }
    }

    pub const fn added(self) -> ChangeTick {
        self.added
    }

    pub const fn changed(self) -> ChangeTick {
        self.changed
    }

    pub fn set_changed(&mut self, tick: ChangeTick) {
        self.changed = tick;
    }

    pub fn is_added(self, window: ChangeTickWindow) -> bool {
        self.added
            .is_newer_than(window.last_run(), window.this_run())
    }

    pub fn is_changed(self, window: ChangeTickWindow) -> bool {
        self.changed
            .is_newer_than(window.last_run(), window.this_run())
    }
}
