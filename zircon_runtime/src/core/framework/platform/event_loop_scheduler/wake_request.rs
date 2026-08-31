use std::time::Instant;

use super::{EventLoopClockDomain, EventLoopWakeSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventLoopWakeRequest {
    source: EventLoopWakeSource,
    domain: EventLoopClockDomain,
    deadline: Instant,
}

impl EventLoopWakeRequest {
    pub const fn immediate(source: EventLoopWakeSource, now: Instant) -> Self {
        Self::at(source, EventLoopClockDomain::Monotonic, now)
    }

    pub const fn at(
        source: EventLoopWakeSource,
        domain: EventLoopClockDomain,
        deadline: Instant,
    ) -> Self {
        Self {
            source,
            domain,
            deadline,
        }
    }

    pub const fn source(self) -> EventLoopWakeSource {
        self.source
    }

    pub const fn domain(self) -> EventLoopClockDomain {
        self.domain
    }

    pub const fn deadline(self) -> Instant {
        self.deadline
    }
}
