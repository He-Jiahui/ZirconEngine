use std::time::Instant;

use super::{WindowCommandHeader, WindowCommandTerminal, WindowObservedGeneration};

/// An immediate acknowledgement from the host command broker. It means the
/// command entered the broker, not that the requested state was applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowCommandAccepted {
    header: WindowCommandHeader,
    accepted_at: Instant,
}

impl WindowCommandAccepted {
    pub(crate) const fn new(header: WindowCommandHeader, accepted_at: Instant) -> Self {
        Self {
            header,
            accepted_at,
        }
    }

    pub const fn header(self) -> WindowCommandHeader {
        self.header
    }

    pub const fn accepted_at(self) -> Instant {
        self.accepted_at
    }
}

/// The exact once-only terminal result for an accepted command. `effective`
/// remains mandatory for every outcome so callers cannot mistake rejection,
/// cancellation, or backend failure for an unknown final window state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowCommandReceipt<Effective, Failure> {
    header: WindowCommandHeader,
    observed_generation: WindowObservedGeneration,
    effective: Effective,
    terminal: WindowCommandTerminal<Failure>,
}

impl<Effective, Failure> WindowCommandReceipt<Effective, Failure> {
    pub(crate) const fn new(
        header: WindowCommandHeader,
        observed_generation: WindowObservedGeneration,
        effective: Effective,
        terminal: WindowCommandTerminal<Failure>,
    ) -> Self {
        Self {
            header,
            observed_generation,
            effective,
            terminal,
        }
    }

    pub const fn header(&self) -> WindowCommandHeader {
        self.header
    }

    pub const fn observed_generation(&self) -> WindowObservedGeneration {
        self.observed_generation
    }

    pub const fn effective(&self) -> &Effective {
        &self.effective
    }

    pub const fn terminal(&self) -> &WindowCommandTerminal<Failure> {
        &self.terminal
    }

    pub fn into_effective(self) -> Effective {
        self.effective
    }
}
