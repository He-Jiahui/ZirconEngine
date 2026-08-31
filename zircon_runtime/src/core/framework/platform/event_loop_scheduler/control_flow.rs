use std::time::Instant;

use super::EventLoopClockDomain;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopControlFlow {
    Poll,
    Wait,
    WaitUntil {
        domain: EventLoopClockDomain,
        deadline: Instant,
    },
}
